//! Inner workings of `smeagol`.

mod impls;
mod store;
mod util;

pub use self::store::{NodeTemplate, Store};
use packed_simd::{u16x16, u8x8};

/// The maximum level a node can have.
pub const MAX_LEVEL: u8 = 64;

/// The index of a node in the store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Index(u32);

/// A unique identifier referring to a node in the store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId {
    index: Index,
}

/// Base representation of a node.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NodeBase {
    /// A level three node.
    LevelThree {
        /// The 8 by 8 board.
        board: u8x8,
    },
    /// A level four node.
    LevelFour {
        /// The 16 by 16 board.
        board: u16x16,
    },
    /// A node with a level greater than four.
    Interior {
        /// The northwest child node.
        nw: NodeId,
        /// The northeast child node.
        ne: NodeId,
        /// The southwest child node.
        sw: NodeId,
        /// The southeast child node.
        se: NodeId,
    },
}

/// An immutable quadtree representation of a Life grid.
#[derive(Clone, Debug)]
pub struct Node {
    /// The base representation of the node.
    pub base: NodeBase,
    /// The level of the node.
    pub level: u8,
    /// The number of alive cells in the node.
    pub population: u128,
}

/// The four quadrants of a node.
pub enum Quadrant {
    /// The northwest quadrant.
    Northwest,
    /// The northeast quadrant.
    Northeast,
    /// The southwest quadrant.
    Southwest,
    /// The southeast quadrant.
    Southeast,
}
