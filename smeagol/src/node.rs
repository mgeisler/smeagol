mod impls;
mod store;

pub use self::store::{NodeTemplate, Store};
use packed_simd::{u16x16, u8x8};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Index(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId {
    index: Index,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NodeBase {
    LevelThree {
        board: u8x8,
    },
    LevelFour {
        board: u16x16,
    },
    Interior {
        nw: NodeId,
        ne: NodeId,
        sw: NodeId,
        se: NodeId,
    },
}

#[derive(Clone, Debug)]
pub struct Node {
    pub base: NodeBase,
    pub level: u8,
    pub population: u128,
}

fn center(row: u16) -> u8 {
    (row >> 4) as u8
}
