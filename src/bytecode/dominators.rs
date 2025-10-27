/// Dominator tree computation and analysis
///
/// A block D dominates block B if every path from entry to B must go through D.
/// The dominator tree represents these relationships efficiently.
use super::cfg::{BlockId, ControlFlowGraph};
use std::collections::{HashMap, HashSet};

/// Dominator tree - represents dominance relationships between basic blocks
#[derive(Debug, Clone)]
pub struct DominatorTree {
    /// Immediate dominator for each block (idom)
    /// The immediate dominator of B is the unique node that strictly dominates B
    /// but does not strictly dominate any other node that strictly dominates B
    pub idom: HashMap<BlockId, BlockId>,

    /// Children in the dominator tree (blocks immediately dominated by this block)
    pub children: HashMap<BlockId, Vec<BlockId>>,

    /// The entry block (root of dominator tree)
    pub entry: BlockId,
}

impl DominatorTree {
    /// Compute the dominator tree using the iterative algorithm
    /// Based on Cooper, Harvey, and Kennedy's "A Simple, Fast Dominance Algorithm"
    pub fn compute(cfg: &ControlFlowGraph) -> Self {
        if cfg.blocks.is_empty() {
            return Self {
                idom: HashMap::new(),
                children: HashMap::new(),
                entry: BlockId(0),
            };
        }

        let entry = cfg.entry_block;

        // Step 1: Compute reverse postorder for efficient iteration
        let rpo = Self::reverse_postorder(cfg, entry);
        let rpo_index: HashMap<BlockId, usize> = rpo
            .iter()
            .enumerate()
            .map(|(i, &block)| (block, i))
            .collect();

        // Step 2: Initialize immediate dominators
        // idom[entry] = entry (by definition)
        let mut idom: HashMap<BlockId, BlockId> = HashMap::new();
        idom.insert(entry, entry);

        // Step 3: Iteratively compute immediate dominators
        let mut changed = true;
        while changed {
            changed = false;

            // Process blocks in reverse postorder (except entry)
            for &block_id in rpo.iter().skip(1) {
                let block = cfg.get_block(block_id).unwrap();

                // Find the first processed predecessor
                let mut new_idom = None;
                for &pred_id in &block.predecessors {
                    if idom.contains_key(&pred_id) {
                        new_idom = Some(pred_id);
                        break;
                    }
                }

                if let Some(mut new_idom_id) = new_idom {
                    // For all other predecessors
                    for &pred_id in &block.predecessors {
                        if pred_id != new_idom_id && idom.contains_key(&pred_id) {
                            // Find common dominator
                            new_idom_id = Self::intersect(&idom, &rpo_index, pred_id, new_idom_id);
                        }
                    }

                    // Update if changed
                    if idom.get(&block_id) != Some(&new_idom_id) {
                        idom.insert(block_id, new_idom_id);
                        changed = true;
                    }
                }
            }
        }

        // Step 4: Build children map from idom
        let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for (&child, &parent) in &idom {
            if child != parent {
                // Don't add entry as its own child
                children.entry(parent).or_default().push(child);
            }
        }

        Self {
            idom,
            children,
            entry,
        }
    }

    /// Compute reverse postorder traversal of the CFG
    fn reverse_postorder(cfg: &ControlFlowGraph, entry: BlockId) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();

        fn dfs(
            cfg: &ControlFlowGraph,
            block_id: BlockId,
            visited: &mut HashSet<BlockId>,
            postorder: &mut Vec<BlockId>,
        ) {
            if visited.contains(&block_id) {
                return;
            }
            visited.insert(block_id);

            if let Some(block) = cfg.get_block(block_id) {
                for &succ in &block.successors {
                    dfs(cfg, succ, visited, postorder);
                }
            }

            postorder.push(block_id);
        }

        dfs(cfg, entry, &mut visited, &mut postorder);
        postorder.reverse();
        postorder
    }

    /// Find the common dominator of two blocks
    fn intersect(
        idom: &HashMap<BlockId, BlockId>,
        rpo_index: &HashMap<BlockId, usize>,
        mut b1: BlockId,
        mut b2: BlockId,
    ) -> BlockId {
        while b1 != b2 {
            while rpo_index[&b1] > rpo_index[&b2] {
                b1 = idom[&b1];
            }
            while rpo_index[&b2] > rpo_index[&b1] {
                b2 = idom[&b2];
            }
        }
        b1
    }

    /// Check if block `dominator` dominates block `dominated`
    pub fn dominates(&self, dominator: BlockId, dominated: BlockId) -> bool {
        if dominator == dominated {
            return true;
        }

        let mut current = dominated;
        while let Some(&idom) = self.idom.get(&current) {
            if idom == current {
                // Reached the entry (which dominates itself)
                break;
            }
            if idom == dominator {
                return true;
            }
            current = idom;
        }

        false
    }

    /// Check if block `dominator` strictly dominates block `dominated`
    /// (dominates but is not equal to)
    pub fn strictly_dominates(&self, dominator: BlockId, dominated: BlockId) -> bool {
        dominator != dominated && self.dominates(dominator, dominated)
    }

    /// Get all blocks dominated by the given block
    pub fn dominated_by(&self, dominator: BlockId) -> HashSet<BlockId> {
        let mut result = HashSet::new();
        result.insert(dominator);

        let mut worklist = vec![dominator];
        while let Some(block) = worklist.pop() {
            if let Some(children) = self.children.get(&block) {
                for &child in children {
                    if result.insert(child) {
                        worklist.push(child);
                    }
                }
            }
        }

        result
    }

    /// Get the immediate dominator of a block
    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied().filter(|&idom| idom != block)
    }

    /// Print the dominator tree in a human-readable format
    pub fn print_debug(&self) {
        println!("Dominator Tree:");
        println!("  Entry Block: {:?}", self.entry);
        println!();

        println!("Immediate Dominators:");
        let mut blocks: Vec<_> = self.idom.keys().collect();
        blocks.sort();
        for &block in blocks {
            let idom = self.idom[&block];
            if block != idom {
                println!("  idom({:?}) = {:?}", block, idom);
            }
        }
        println!();

        println!("Dominator Tree Structure:");
        self.print_tree(self.entry, 0);
    }

    fn print_tree(&self, block: BlockId, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}{:?}", indent, block);

        if let Some(children) = self.children.get(&block) {
            let mut children = children.clone();
            children.sort();
            for &child in &children {
                self.print_tree(child, depth + 1);
            }
        }
    }

    /// Compute the dominance frontier of a block
    /// DF(X) = set of blocks where X's dominance stops
    /// (blocks that have a predecessor dominated by X, but are not strictly dominated by X)
    pub fn dominance_frontier(&self, cfg: &ControlFlowGraph, block: BlockId) -> HashSet<BlockId> {
        let mut frontier = HashSet::new();
        let dominated = self.dominated_by(block);

        // For each block Y dominated by X
        for &y in &dominated {
            if let Some(y_block) = cfg.get_block(y) {
                // For each successor S of Y
                for &s in &y_block.successors {
                    // If S is not strictly dominated by X, it's in the frontier
                    if !self.strictly_dominates(block, s) {
                        frontier.insert(s);
                    }
                }
            }
        }

        frontier
    }
}

/// Post-dominator tree - represents post-dominance relationships between basic blocks
/// A block X post-dominates block Y if all paths from Y to any exit must go through X
#[derive(Debug, Clone)]
pub struct PostDominatorTree {
    /// Immediate post-dominator for each block (ipdom)
    /// The immediate post-dominator of B is the unique node that strictly post-dominates B
    /// but does not strictly post-dominate any other node that strictly post-dominates B
    pub ipdom: HashMap<BlockId, BlockId>,

    /// Children in the post-dominator tree (blocks immediately post-dominated by this block)
    pub children: HashMap<BlockId, Vec<BlockId>>,

    /// Virtual exit block that post-dominates all actual exits
    pub virtual_exit: BlockId,

    /// Actual exit blocks (blocks with no successors or ending in Return)
    pub exit_blocks: HashSet<BlockId>,
}

impl PostDominatorTree {
    /// Compute the post-dominator tree using the iterative algorithm
    /// Similar to dominator tree but works backwards from exits
    pub fn compute(cfg: &ControlFlowGraph) -> Self {
        if cfg.blocks.is_empty() {
            return Self {
                ipdom: HashMap::new(),
                children: HashMap::new(),
                virtual_exit: BlockId(usize::MAX),
                exit_blocks: HashSet::new(),
            };
        }

        // Step 1: Identify exit blocks (blocks with no successors)
        let mut exit_blocks = HashSet::new();
        for block in &cfg.blocks {
            if block.successors.is_empty() {
                exit_blocks.insert(block.id);
            }
        }

        // If no exit blocks found, use the last block as exit
        if exit_blocks.is_empty()
            && let Some(last_block) = cfg.blocks.last()
        {
            exit_blocks.insert(last_block.id);
        }

        // Step 2: Create a virtual exit block that all actual exits lead to
        let virtual_exit = BlockId(usize::MAX);

        // Step 3: Compute reverse postorder from exits (postorder of reverse CFG)
        let rpo = Self::reverse_postorder_from_exits(cfg, &exit_blocks);
        let rpo_index: HashMap<BlockId, usize> = rpo
            .iter()
            .enumerate()
            .map(|(i, &block)| (block, i))
            .collect();

        // Step 4: Initialize immediate post-dominators
        // ipdom[virtual_exit] = virtual_exit (by definition)
        let mut ipdom: HashMap<BlockId, BlockId> = HashMap::new();
        ipdom.insert(virtual_exit, virtual_exit);

        // All exit blocks are immediately post-dominated by the virtual exit
        for &exit in &exit_blocks {
            ipdom.insert(exit, virtual_exit);
        }

        // Step 5: Iteratively compute immediate post-dominators
        let mut changed = true;
        while changed {
            changed = false;

            // Process blocks in reverse postorder (except exits)
            for &block_id in &rpo {
                if exit_blocks.contains(&block_id) {
                    continue; // Skip exit blocks - already initialized
                }

                let block = cfg.get_block(block_id).unwrap();

                // Find the first processed successor
                let mut new_ipdom = None;
                for &succ_id in &block.successors {
                    if ipdom.contains_key(&succ_id) {
                        new_ipdom = Some(succ_id);
                        break;
                    }
                }

                if let Some(mut new_ipdom_id) = new_ipdom {
                    // For all other successors
                    for &succ_id in &block.successors {
                        if succ_id != new_ipdom_id && ipdom.contains_key(&succ_id) {
                            // Find common post-dominator
                            new_ipdom_id =
                                Self::intersect(&ipdom, &rpo_index, succ_id, new_ipdom_id);
                        }
                    }

                    // Update if changed
                    if ipdom.get(&block_id) != Some(&new_ipdom_id) {
                        ipdom.insert(block_id, new_ipdom_id);
                        changed = true;
                    }
                }
            }
        }

        // Step 6: Build children map from ipdom
        let mut children: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for (&child, &parent) in &ipdom {
            if child != parent && parent != virtual_exit {
                // Don't add virtual exit relationships to children
                children.entry(parent).or_default().push(child);
            }
        }

        Self {
            ipdom,
            children,
            virtual_exit,
            exit_blocks,
        }
    }

    /// Compute reverse postorder from exit blocks (for post-dominator analysis)
    /// This is essentially a postorder traversal of the reverse CFG
    fn reverse_postorder_from_exits(
        cfg: &ControlFlowGraph,
        exit_blocks: &HashSet<BlockId>,
    ) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();

        fn dfs_reverse(
            cfg: &ControlFlowGraph,
            block_id: BlockId,
            visited: &mut HashSet<BlockId>,
            postorder: &mut Vec<BlockId>,
        ) {
            if visited.contains(&block_id) {
                return;
            }
            visited.insert(block_id);

            if let Some(block) = cfg.get_block(block_id) {
                // Visit predecessors (reverse CFG)
                for &pred in &block.predecessors {
                    dfs_reverse(cfg, pred, visited, postorder);
                }
            }

            postorder.push(block_id);
        }

        // Start DFS from all exit blocks
        for &exit in exit_blocks {
            dfs_reverse(cfg, exit, &mut visited, &mut postorder);
        }

        postorder.reverse();
        postorder
    }

    /// Find the common post-dominator of two blocks
    fn intersect(
        ipdom: &HashMap<BlockId, BlockId>,
        rpo_index: &HashMap<BlockId, usize>,
        mut b1: BlockId,
        mut b2: BlockId,
    ) -> BlockId {
        while b1 != b2 {
            // If b1 is not in rpo_index, it means we haven't processed it yet
            // In this case, move b1 up the post-dominator tree
            while rpo_index.get(&b1).copied().unwrap_or(usize::MAX)
                > rpo_index.get(&b2).copied().unwrap_or(usize::MAX)
            {
                if let Some(&next) = ipdom.get(&b1) {
                    b1 = next;
                } else {
                    return b2;
                }
            }
            while rpo_index.get(&b2).copied().unwrap_or(usize::MAX)
                > rpo_index.get(&b1).copied().unwrap_or(usize::MAX)
            {
                if let Some(&next) = ipdom.get(&b2) {
                    b2 = next;
                } else {
                    return b1;
                }
            }
        }
        b1
    }

    /// Check if block `postdom` post-dominates block `postdominated`
    pub fn post_dominates(&self, postdom: BlockId, postdominated: BlockId) -> bool {
        if postdom == postdominated {
            return true;
        }

        let mut current = postdominated;
        while let Some(&ipdom) = self.ipdom.get(&current) {
            if ipdom == current || ipdom == self.virtual_exit {
                // Reached the exit
                break;
            }
            if ipdom == postdom {
                return true;
            }
            current = ipdom;
        }

        false
    }

    /// Check if block `postdom` strictly post-dominates block `postdominated`
    /// (post-dominates but is not equal to)
    pub fn strictly_post_dominates(&self, postdom: BlockId, postdominated: BlockId) -> bool {
        postdom != postdominated && self.post_dominates(postdom, postdominated)
    }

    /// Get the immediate post-dominator of a block
    pub fn immediate_post_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.ipdom
            .get(&block)
            .copied()
            .filter(|&ipdom| ipdom != block && ipdom != self.virtual_exit)
    }

    /// Find the immediate common post-dominator of two blocks
    /// This is the merge point where both branches come together
    pub fn immediate_common_post_dominator(&self, b1: BlockId, b2: BlockId) -> Option<BlockId> {
        // Collect all post-dominators of b1
        let mut b1_postdoms = HashSet::new();
        let mut current = b1;
        while let Some(&ipdom) = self.ipdom.get(&current) {
            if ipdom == current || ipdom == self.virtual_exit {
                break;
            }
            b1_postdoms.insert(ipdom);
            current = ipdom;
        }

        // Check if b2 itself is a post-dominator of b1
        if b1_postdoms.contains(&b2) {
            return Some(b2);
        }

        // Check if b1 itself is a post-dominator of b2 (symmetric case)
        // This ensures the function is symmetric: icpdom(A,B) == icpdom(B,A)
        current = b2;
        while let Some(&ipdom) = self.ipdom.get(&current) {
            if ipdom == current || ipdom == self.virtual_exit {
                break;
            }
            if ipdom == b1 {
                return Some(b1);
            }
            current = ipdom;
        }

        // Neither post-dominates the other - find first common post-dominator
        current = b2;
        while let Some(&ipdom) = self.ipdom.get(&current) {
            if ipdom == current || ipdom == self.virtual_exit {
                break;
            }
            if b1_postdoms.contains(&ipdom) {
                return Some(ipdom);
            }
            current = ipdom;
        }

        None
    }

    /// Print the post-dominator tree in a human-readable format
    pub fn print_debug(&self) {
        println!("Post-Dominator Tree:");
        println!("  Virtual Exit: <exit>");
        println!("  Exit Blocks: {:?}", self.exit_blocks);
        println!();

        println!("Immediate Post-Dominators:");
        let mut blocks: Vec<_> = self.ipdom.keys().collect();
        blocks.sort();
        for &block in blocks {
            let ipdom = self.ipdom[&block];
            if block != ipdom {
                if ipdom == self.virtual_exit {
                    println!("  ipdom({:?}) = <exit>", block);
                } else {
                    println!("  ipdom({:?}) = {:?}", block, ipdom);
                }
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_simple_dominance() {
        // Create a simple CFG for testing
        // This would need actual CFG construction, just a placeholder
    }
}
