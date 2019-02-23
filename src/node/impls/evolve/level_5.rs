use crate::node::*;
use packed_simd::{u16x16, u16x32};

#[derive(Clone, Copy, Debug)]
struct CountsU16x16 {
    low: u16x16,
    mid: u16x16,
    high: u16x16,
}

impl CountsU16x16 {
    fn new() -> Self {
        Self {
            low: u16x16::splat(0),
            mid: u16x16::splat(0),
            high: u16x16::splat(0),
        }
    }

    fn add(&mut self, neighbors: u16x16) {
        // low bit half adder
        let low_carry = self.low & neighbors;
        self.low ^= neighbors;

        // middle bit half adder
        let mid_carry = self.mid & low_carry;
        self.mid ^= low_carry;

        // high bit saturating add
        self.high |= mid_carry;
    }
}

fn rotate_lanes_up_u16x16(board: u16x16) -> u16x16 {
    shuffle!(
        board,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]
    )
}

fn rotate_lanes_down_u16x16(board: u16x16) -> u16x16 {
    shuffle!(
        board,
        [15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
    )
}

fn step_once_u16x16(board: u16x16) -> u16x16 {
    let mut neighbors = CountsU16x16::new();

    // +---+---+---+
    // | * | * | * |
    // +---+---+---+
    // | * |   | * |
    // +---+---+---+
    // | * | * | * |
    // +---+---+---+

    // top row
    neighbors.add(rotate_lanes_down_u16x16(board) >> 1);
    neighbors.add(rotate_lanes_down_u16x16(board));
    neighbors.add(rotate_lanes_down_u16x16(board) << 1);

    // middle row
    neighbors.add(board >> 1);
    neighbors.add(board << 1);

    // bottom row
    neighbors.add(rotate_lanes_up_u16x16(board) >> 1);
    neighbors.add(rotate_lanes_up_u16x16(board));
    neighbors.add(rotate_lanes_up_u16x16(board) << 1);

    // 2 is 010 in binary
    let two_neighbors = !neighbors.high & neighbors.mid & !neighbors.low;
    // 3 is 011 in binary
    let three_neighbors = !neighbors.high & neighbors.mid & neighbors.low;

    // if 2 neighbors, the cell doesn't change
    // if 3 neighbors, the cell is alive
    (two_neighbors & board) | three_neighbors
}

fn jump_u16x16(mut board: u16x16) -> u16x16 {
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board = step_once_u16x16(board);
    board
}

fn step_u16x16(mut board: u16x16, step_log_2: u8) -> u16x16 {
    for _ in 0..(1 << step_log_2) {
        board = step_once_u16x16(board);
    }
    board
}

fn horiz_jump_u16x16(w: u16x16, e: u16x16) -> u16x16 {
    let grid = (w << 8) | (e >> 8);
    jump_u16x16(grid)
}

fn center_jump_u16x16(
    nw_grid: u16x16,
    ne_grid: u16x16,
    sw_grid: u16x16,
    se_grid: u16x16,
) -> u16x16 {
    let grid = center(nw_grid, ne_grid, sw_grid, se_grid);
    jump_u16x16(grid)
}

fn combine_results_u16x16(
    nw_grid: u16x16,
    ne_grid: u16x16,
    sw_grid: u16x16,
    se_grid: u16x16,
) -> u16x16 {
    let nw_grid = nw_grid << 4;
    let nw_grid = shuffle!(
        nw_grid,
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3]
    ) & LEVEL_4_NW_MASK;

    let ne_grid = ne_grid >> 4;
    let ne_grid = shuffle!(
        ne_grid,
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3]
    ) & LEVEL_4_NE_MASK;

    let sw_grid = sw_grid << 4;
    let sw_grid = shuffle!(
        sw_grid,
        [12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    ) & LEVEL_4_SW_MASK;

    let se_grid = se_grid >> 4;
    let se_grid = shuffle!(
        se_grid,
        [12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    ) & LEVEL_4_SE_MASK;

    nw_grid | ne_grid | sw_grid | se_grid
}

fn horiz_u16x16(w: u16x16, e: u16x16) -> u16x16 {
    (w << 8) | (e >> 8)
}

fn vert_u16x16(n: u16x16, s: u16x16) -> u16x16 {
    let n = shuffle!(n, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_UPPER_HALF_MASK;
    let s = shuffle!(s, [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7])
        & LEVEL_4_LOWER_HALF_MASK;
    n | s
}

#[allow(clippy::many_single_char_names)]
pub fn step_level_5(
    store: &mut Store,
    step_log_2: u8,
    nw: NodeId,
    ne: NodeId,
    sw: NodeId,
    se: NodeId,
) -> NodeId {
    let nw_grid = store.node(nw).unwrap_leaf();
    let ne_grid = store.node(ne).unwrap_leaf();
    let sw_grid = store.node(sw).unwrap_leaf();
    let se_grid = store.node(se).unwrap_leaf();

    let a = nw_grid;
    let b = horiz_u16x16(nw_grid, ne_grid);
    let c = ne_grid;
    let d = vert_u16x16(nw_grid, sw_grid);
    let e = center(nw_grid, ne_grid, sw_grid, se_grid);
    let f = vert_u16x16(ne_grid, se_grid);
    let g = sw_grid;
    let h = horiz_u16x16(sw_grid, se_grid);
    let i = se_grid;

    let w = step_u16x16(combine_results_u16x16(a, b, d, e), step_log_2);
    let x = step_u16x16(combine_results_u16x16(b, c, e, f), step_log_2);
    let y = step_u16x16(combine_results_u16x16(d, e, g, h), step_log_2);
    let z = step_u16x16(combine_results_u16x16(e, f, h, i), step_log_2);

    store.create_leaf(combine_results_u16x16(w, x, y, z))
}

#[allow(clippy::many_single_char_names)]
pub fn jump_level_5(store: &mut Store, nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId) -> NodeId {
    let nw_grid = store.node(nw).unwrap_leaf();
    let ne_grid = store.node(ne).unwrap_leaf();
    let sw_grid = store.node(sw).unwrap_leaf();
    let se_grid = store.node(se).unwrap_leaf();

    let left: u16x32 = shuffle!(
        nw_grid,
        sw_grid,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31
        ]
    );
    let left = jump_u16x32(left);

    let right: u16x32 = shuffle!(
        ne_grid,
        se_grid,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31
        ]
    );
    let right = jump_u16x32(right);

    let a = shuffle!(left, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let d = shuffle!(left, [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]);
    let g = shuffle!(left, [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);

    let c = shuffle!(right, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let f = shuffle!(right, [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]);
    let i = shuffle!(right, [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);

    let b = horiz_jump_u16x16(nw_grid, ne_grid);
    let e = center_jump_u16x16(nw_grid, ne_grid, sw_grid, se_grid);
    let h = horiz_jump_u16x16(sw_grid, se_grid);

    let w = jump_u16x16(combine_results_u16x16(a, b, d, e));
    let x = jump_u16x16(combine_results_u16x16(b, c, e, f));
    let y = jump_u16x16(combine_results_u16x16(d, e, g, h));
    let z = jump_u16x16(combine_results_u16x16(e, f, h, i));

    store.create_leaf(combine_results_u16x16(w, x, y, z))
}

#[derive(Clone, Copy, Debug)]
struct CountsU16x32 {
    low: u16x32,
    mid: u16x32,
    high: u16x32,
}

impl CountsU16x32 {
    fn new() -> Self {
        Self {
            low: u16x32::splat(0),
            mid: u16x32::splat(0),
            high: u16x32::splat(0),
        }
    }

    fn add(&mut self, neighbors: u16x32) {
        // low bit half adder
        let low_carry = self.low & neighbors;
        self.low ^= neighbors;

        // middle bit half adder
        let mid_carry = self.mid & low_carry;
        self.mid ^= low_carry;

        // high bit saturating add
        self.high |= mid_carry;
    }
}

fn rotate_lanes_up_u16x32(board: u16x32) -> u16x32 {
    shuffle!(
        board,
        [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 0
        ]
    )
}

fn rotate_lanes_down_u16x32(board: u16x32) -> u16x32 {
    shuffle!(
        board,
        [
            31, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30
        ]
    )
}

fn step_once_u16x32(board: u16x32) -> u16x32 {
    let mut neighbors = CountsU16x32::new();

    // +---+---+---+
    // | * | * | * |
    // +---+---+---+
    // | * |   | * |
    // +---+---+---+
    // | * | * | * |
    // +---+---+---+

    // top row
    neighbors.add(rotate_lanes_down_u16x32(board) >> 1);
    neighbors.add(rotate_lanes_down_u16x32(board));
    neighbors.add(rotate_lanes_down_u16x32(board) << 1);

    // middle row
    neighbors.add(board >> 1);
    neighbors.add(board << 1);

    // bottom row
    neighbors.add(rotate_lanes_up_u16x32(board) >> 1);
    neighbors.add(rotate_lanes_up_u16x32(board));
    neighbors.add(rotate_lanes_up_u16x32(board) << 1);

    // 2 is 010 in binary
    let two_neighbors = !neighbors.high & neighbors.mid & !neighbors.low;
    // 3 is 011 in binary
    let three_neighbors = !neighbors.high & neighbors.mid & neighbors.low;

    // if 2 neighbors, the cell doesn't change
    // if 3 neighbors, the cell is alive
    (two_neighbors & board) | three_neighbors
}

fn jump_u16x32(mut board: u16x32) -> u16x32 {
    board = step_once_u16x32(board);
    board = step_once_u16x32(board);
    board = step_once_u16x32(board);
    board = step_once_u16x32(board);
    board
}
