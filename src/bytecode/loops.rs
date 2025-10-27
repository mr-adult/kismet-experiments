/// Loop detection and analysis
///
/// Identifies natural loops in the control flow graph using back edges
use super::cfg::{BlockId, ControlFlowGraph};
use super::dominators::DominatorTree;
use std::collections::{HashSet, VecDeque};

/// A natural loop in the control flow graph
#[derive(Debug, Clone)]
pub struct Loop {
    /// The header block (entry point of the loop)
    pub header: BlockId,

    /// All blocks that are part of this loop
    pub blocks: HashSet<BlockId>,

    /// Back edges that form this loop (from latch to header)
    pub back_edges: Vec<(BlockId, BlockId)>,

    /// Exit blocks (blocks in the loop with successors outside)
    pub exit_blocks: HashSet<BlockId>,

    /// The parent loop (if this is a nested loop)
    pub parent: Option<usize>,

    /// Child loops (nested inside this loop)
    pub children: Vec<usize>,
}

impl Loop {
    fn new(header: BlockId) -> Self {
        Self {
            header,
            blocks: HashSet::new(),
            back_edges: Vec::new(),
            exit_blocks: HashSet::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Check if this loop is nested inside another loop
    pub fn is_nested(&self) -> bool {
        self.parent.is_some()
    }

    /// Get the depth of nesting (0 = outermost loop)
    pub fn nesting_depth(&self, all_loops: &[Loop]) -> usize {
        let mut depth = 0;
        let mut current = self.parent;
        while let Some(parent_idx) = current {
            depth += 1;
            current = all_loops[parent_idx].parent;
        }
        depth
    }
}

/// Collection of all loops in a function
#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub loops: Vec<Loop>,
}

impl LoopInfo {
    /// Detect all natural loops in the CFG
    pub fn analyze(cfg: &ControlFlowGraph, dom_tree: &DominatorTree) -> Self {
        let mut loops = Vec::new();

        // Step 1: Find all back edges
        // A back edge is an edge from node B to node H where H dominates B
        let back_edges = Self::find_back_edges(cfg, dom_tree);

        // Step 2: For each back edge, construct the natural loop
        let mut loop_map: std::collections::HashMap<BlockId, usize> =
            std::collections::HashMap::new();

        for (latch, header) in back_edges {
            // Check if we already have a loop with this header
            let loop_idx = if let Some(&idx) = loop_map.get(&header) {
                idx
            } else {
                let idx = loops.len();
                loops.push(Loop::new(header));
                loop_map.insert(header, idx);
                idx
            };

            // Add the back edge
            loops[loop_idx].back_edges.push((latch, header));

            // Find all blocks in the natural loop
            let loop_blocks = Self::find_natural_loop(cfg, header, latch);
            loops[loop_idx].blocks.extend(loop_blocks);
        }

        // Step 3: Find exit blocks for each loop
        for loop_info in &mut loops {
            loop_info.exit_blocks = Self::find_exit_blocks(cfg, &loop_info.blocks);
        }

        // Step 4: Build loop nesting tree
        Self::build_loop_tree(&mut loops);

        Self { loops }
    }

    /// Find all back edges in the CFG
    /// A back edge (B -> H) exists if H dominates B
    fn find_back_edges(
        cfg: &ControlFlowGraph,
        dom_tree: &DominatorTree,
    ) -> Vec<(BlockId, BlockId)> {
        let mut back_edges = Vec::new();

        for block in &cfg.blocks {
            for &succ in &block.successors {
                // If successor dominates this block, it's a back edge
                if dom_tree.dominates(succ, block.id) {
                    back_edges.push((block.id, succ));
                }
            }
        }

        back_edges
    }

    /// Find all blocks in the natural loop formed by back edge (latch -> header)
    /// The natural loop consists of:
    /// - The header
    /// - All nodes that can reach the latch without going through the header
    fn find_natural_loop(
        cfg: &ControlFlowGraph,
        header: BlockId,
        latch: BlockId,
    ) -> HashSet<BlockId> {
        let mut loop_blocks = HashSet::new();
        loop_blocks.insert(header);
        loop_blocks.insert(latch);

        // Work backwards from the latch to find all blocks that can reach it
        let mut worklist = VecDeque::new();
        worklist.push_back(latch);

        while let Some(block_id) = worklist.pop_front() {
            if let Some(block) = cfg.get_block(block_id) {
                for &pred in &block.predecessors {
                    // Don't go past the header
                    if pred != header && loop_blocks.insert(pred) {
                        worklist.push_back(pred);
                    }
                }
            }
        }

        loop_blocks
    }

    /// Find exit blocks for a loop
    /// An exit block is a block in the loop with a successor outside the loop
    fn find_exit_blocks(
        cfg: &ControlFlowGraph,
        loop_blocks: &HashSet<BlockId>,
    ) -> HashSet<BlockId> {
        let mut exit_blocks = HashSet::new();

        for &block_id in loop_blocks {
            if let Some(block) = cfg.get_block(block_id) {
                for &succ in &block.successors {
                    if !loop_blocks.contains(&succ) {
                        exit_blocks.insert(block_id);
                        break;
                    }
                }
            }
        }

        exit_blocks
    }

    /// Build the loop nesting tree
    /// A loop L1 is nested in L2 if all blocks of L1 are contained in L2
    fn build_loop_tree(loops: &mut Vec<Loop>) {
        for i in 0..loops.len() {
            // Check if loop i is nested in any other loop
            let mut potential_parents = Vec::new();

            for j in 0..loops.len() {
                if i == j {
                    continue;
                }

                // Check if loop i is nested in loop j
                let is_nested = loops[i].header != loops[j].header
                    && loops[i].blocks.is_subset(&loops[j].blocks);

                if is_nested {
                    potential_parents.push(j);
                }
            }

            // Find the innermost parent (smallest loop that contains this one)
            if !potential_parents.is_empty() {
                let mut innermost = potential_parents[0];
                let mut min_size = loops[innermost].blocks.len();

                for &parent_idx in &potential_parents {
                    let size = loops[parent_idx].blocks.len();
                    if size < min_size {
                        min_size = size;
                        innermost = parent_idx;
                    }
                }

                loops[i].parent = Some(innermost);
            }
        }

        // Build children relationships
        for i in 0..loops.len() {
            if let Some(parent_idx) = loops[i].parent {
                loops[parent_idx].children.push(i);
            }
        }
    }

    /// Get the loop that contains a given block, if any
    pub fn get_loop_for_block(&self, block: BlockId) -> Option<&Loop> {
        // Find the innermost loop containing this block
        let mut result = None;
        let mut min_size = usize::MAX;

        for loop_info in &self.loops {
            if loop_info.blocks.contains(&block) && loop_info.blocks.len() < min_size {
                result = Some(loop_info);
                min_size = loop_info.blocks.len();
            }
        }

        result
    }

    /// Check if a block is a loop header
    pub fn is_loop_header(&self, block: BlockId) -> bool {
        self.loops.iter().any(|l| l.header == block)
    }

    /// Print loop information
    pub fn print_debug(&self) {
        println!("Loop Analysis:");
        println!("  Total Loops: {}", self.loops.len());
        println!();

        for (i, loop_info) in self.loops.iter().enumerate() {
            println!("Loop {}:", i);
            println!("  Header: {:?}", loop_info.header);
            println!("  Blocks: {:?}", {
                let mut blocks: Vec<_> = loop_info.blocks.iter().collect();
                blocks.sort();
                blocks
            });
            println!("  Back Edges: {:?}", loop_info.back_edges);
            println!("  Exit Blocks: {:?}", {
                let mut exits: Vec<_> = loop_info.exit_blocks.iter().collect();
                exits.sort();
                exits
            });
            if let Some(parent) = loop_info.parent {
                println!("  Parent Loop: {}", parent);
            }
            if !loop_info.children.is_empty() {
                println!("  Child Loops: {:?}", loop_info.children);
            }
            println!("  Nesting Depth: {}", loop_info.nesting_depth(&self.loops));
            println!();
        }
    }
}
