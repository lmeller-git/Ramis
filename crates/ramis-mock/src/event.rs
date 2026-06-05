//! Contains useful branching factors

use ramis_core::generate_static_event;

generate_static_event! {
    /// A Tree with a branching factor of 1, i.e. a line
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum UnitBranch {
        /// The single branchpoint
        Value,
    }
}

generate_static_event! {
    /// A BinaryTree
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum BinaryBranch {
        /// One branch in the BinaryTree
        Left,
        /// The other branch in the BinaryTree
        Right,
    }
}

generate_static_event! {
    /// A Ternary tree
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    pub enum TernaryBranch {
        /// One branch in the Ternary Tree
        Left,
        /// Another branch in the Ternary Tree
        Middle,
        /// Another branch in the Ternary Tree
        Right
    }
}
