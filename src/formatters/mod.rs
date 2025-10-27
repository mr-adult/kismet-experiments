pub mod asm;
pub mod cpp;
pub mod theme;

#[derive(Default, Debug, Clone, Copy)]
pub struct FormattingOptions {
    /// Prefix blocks with block ID comments (e.g., `// block: BlockId(0)`)
    pub show_block_ids: bool,
    /// Prefix statements with inline bytecode offset comments (e.g., `/* 0x1A3 */ variable = value;`)
    pub show_bytecode_offsets: bool,
    /// Show terminator expressions as comments at the end of basic blocks
    pub show_terminator_exprs: bool,
}
