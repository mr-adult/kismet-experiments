use clap::{Parser, ValueEnum};
use std::fs;

mod bytecode;
mod dot;
mod formatters;

use crate::{
    bytecode::{
        address_index::AddressIndex,
        cfg::{ControlFlowGraph, Terminator},
        dominators::{DominatorTree, PostDominatorTree},
        expr::{ExprKind, collect_referenced_offsets},
        loops::LoopInfo,
        parser::ScriptParser,
        reader::ScriptReader,
        structured::PhoenixStructurer,
    },
    formatters::{asm::AsmFormatter, cpp::CppFormatter},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Cpp,
    Asm,
    Analyze,
    Structured,
    Dot,
    Cfg,
}

#[derive(Parser, Debug)]
struct Args {
    /// Path to the JMAP file
    jmap_file: String,

    /// Filter functions by name (optional)
    #[arg(short, long)]
    filter: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "cpp")]
    format: OutputFormat,

    /// Show block ID comments in structured output
    #[arg(long)]
    show_block_ids: bool,

    /// Show bytecode offset comments in structured output
    #[arg(long)]
    show_bytecode_offsets: bool,

    /// Show terminator expressions as comments in structured output
    #[arg(long)]
    show_terminator_exprs: bool,
}

fn main() {
    let args = Args::parse();

    println!("Loading JMAP file: {}", args.jmap_file);

    let jmap_data = match fs::read_to_string(&args.jmap_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let jmap: jmap::Jmap = match serde_json::from_str(&jmap_data) {
        Ok(jmap) => jmap,
        Err(e) => {
            eprintln!("Error parsing JMAP JSON: {}", e);
            std::process::exit(1);
        }
    };

    println!("Loaded JMAP with {} objects", jmap.objects.len());

    // Build address index for resolving object and property references
    let address_index = AddressIndex::new(&jmap);
    println!(
        "Built address index with {} entries",
        address_index.object_index.len() + address_index.property_index.len()
    );

    // Count and disassemble functions
    let mut function_count = 0;
    let mut disassembled_count = 0;

    for (name, obj) in &jmap.objects {
        if let jmap::ObjectType::Function(func) = obj {
            function_count += 1;
            if name.contains("ExecuteUbergraph") {
                continue;
            }
            // if func.r#struct.script.len() < 10000 {
            //     continue;
            // }

            // Apply filter if specified
            if let Some(ref filter_str) = args.filter
                && !name.contains(filter_str)
            {
                continue;
            }

            let script = &func.r#struct.script;

            if script.is_empty() {
                continue;
            }

            disassembled_count += 1;

            println!("\n{}", "=".repeat(80));
            println!("Function: {}", name);
            println!("Address: {:?}", func.r#struct.object.address);
            println!("Flags: {:?}", func.function_flags);
            println!("Script size: {} bytes", script.len());
            println!("{}\n", "=".repeat(80));

            // Parse bytecode to IR
            let reader = ScriptReader::new(
                script,
                jmap.names.as_ref().expect("name map is required"),
                &address_index,
            );
            let mut parser = ScriptParser::new(reader);
            let expressions = parser.parse_all();

            // Collect all referenced bytecode offsets
            let referenced_offsets = collect_referenced_offsets(&expressions);

            // Format based on output type
            match args.format {
                OutputFormat::Asm => {
                    let mut formatter = AsmFormatter::new(&address_index, referenced_offsets);
                    formatter.format(&expressions);
                }
                OutputFormat::Cpp => {
                    let mut formatter = CppFormatter::new(&address_index, referenced_offsets);
                    formatter.format(&expressions);
                }
                OutputFormat::Analyze => {
                    // Build and display the Control Flow Graph
                    let cfg = ControlFlowGraph::from_expressions(&expressions);
                    cfg.print_debug(&expressions, &address_index);

                    // Compute and display dominator tree
                    println!("\n{}", "=".repeat(80));
                    let dom_tree = DominatorTree::compute(&cfg);
                    dom_tree.print_debug();

                    // Detect and display loops
                    println!("\n{}", "=".repeat(80));
                    let loop_info = LoopInfo::analyze(&cfg, &dom_tree);
                    loop_info.print_debug();

                    // Compute and display post-dominator tree
                    println!("\n{}", "=".repeat(80));
                    let post_dom_tree = PostDominatorTree::compute(&cfg);
                    post_dom_tree.print_debug();

                    // Compute and display structured statements
                    println!("\n{}", "=".repeat(80));
                    let structurer = PhoenixStructurer::new(&cfg, &loop_info);
                    if let Some(structured) = structurer.structure() {
                        structured.print(&address_index);
                    } else {
                        eprintln!("Failed to fully structure the control flow");
                    }
                }
                OutputFormat::Structured => {
                    // Build CFG and analysis
                    let cfg = ControlFlowGraph::from_expressions(&expressions);
                    let dom_tree = DominatorTree::compute(&cfg);
                    let loop_info = LoopInfo::analyze(&cfg, &dom_tree);

                    // Structure the control flow
                    let structurer = PhoenixStructurer::new(&cfg, &loop_info);

                    if let Some(structured) = structurer.structure() {
                        structured.print(&address_index);
                    } else {
                        eprintln!("Failed to fully structure the control flow");
                    }
                }
                OutputFormat::Dot => {
                    // Build CFG and generate DOT graph
                    let cfg = ControlFlowGraph::from_expressions(&expressions);
                    let graph = cfg.to_dot(&expressions, &address_index);

                    let mut output = String::new();
                    graph
                        .write(&mut output)
                        .expect("Failed to generate DOT output");

                    render_dot_and_open(output);
                }
                OutputFormat::Cfg => {
                    // Build CFG and print in flat format with block IDs
                    let cfg = ControlFlowGraph::from_expressions(&expressions);

                    // Print blocks in order
                    for block in &cfg.blocks {
                        // Print block header as a styled label using Theme
                        println!(
                            "{}:",
                            formatters::theme::Theme::label(format!("Block_{}", block.id.0))
                        );

                        // Print statements using CppFormatter, filtering out execution flow ops
                        let mut formatter =
                            CppFormatter::new(&address_index, referenced_offsets.clone());
                        formatter.set_indent_level(1);
                        for stmt in &block.statements {
                            match &stmt.kind {
                                ExprKind::PushExecutionFlow { .. }
                                | ExprKind::PopExecutionFlow
                                | ExprKind::PopExecutionFlowIfNot { .. } => {
                                    continue;
                                }
                                _ => {
                                    formatter.format_statement(stmt);
                                }
                            }
                        }

                        // Print CFG terminator instead of expression terminator
                        match &block.terminator {
                            Terminator::Goto { target } => {
                                println!(
                                    "    goto {};",
                                    formatters::theme::Theme::label(format!("Block_{}", target.0))
                                );
                            }
                            Terminator::Branch {
                                condition,
                                true_target,
                                false_target,
                            } => {
                                let cond_str = formatter.format_expr_inline(
                                    condition,
                                    &formatters::cpp::FormatContext::This,
                                );
                                println!(
                                    "    if ({}) goto {}; else goto {};",
                                    cond_str,
                                    formatters::theme::Theme::label(format!(
                                        "Block_{}",
                                        true_target.0
                                    )),
                                    formatters::theme::Theme::label(format!(
                                        "Block_{}",
                                        false_target.0
                                    ))
                                );
                            }
                            Terminator::DynamicJump => {
                                println!("    // dynamic jump");
                            }
                            Terminator::Return(expr) => {
                                let ret_str = formatter.format_expr_inline(
                                    expr,
                                    &formatters::cpp::FormatContext::This,
                                );
                                println!("    return {};", ret_str);
                            }
                            Terminator::None => unreachable!(),
                        }

                        println!();
                    }
                }
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("Summary:");
    println!("  Total functions: {}", function_count);
    println!("  Disassembled: {}", disassembled_count);
    println!("{}", "=".repeat(80));
}

fn render_dot_and_open(dot: String) {
    let dot_path = "/tmp/graph.dot";
    let svg_path = "/tmp/graph.svg";

    if let Err(e) = std::fs::write(dot_path, &dot) {
        eprintln!("Failed to write DOT file: {}", e);
    } else {
        eprintln!("Graph saved to: {}", dot_path);

        // Generate SVG with dot
        match std::process::Command::new("dot")
            .arg("-Tsvg")
            .arg(dot_path)
            .arg("-o")
            .arg(svg_path)
            .status()
        {
            Ok(status) if status.success() => {
                eprintln!("SVG generated: {}", svg_path);

                // Open in Firefox
                match std::process::Command::new("firefox").arg(svg_path).spawn() {
                    Ok(_) => eprintln!("Opened in Firefox"),
                    Err(e) => eprintln!("Failed to open Firefox: {}", e),
                }
            }
            Ok(status) => eprintln!("dot command failed with status: {}", status),
            Err(e) => eprintln!("Failed to run dot: {}", e),
        }
    }
}
