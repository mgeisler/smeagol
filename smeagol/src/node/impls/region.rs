use crate::node::{util, NodeBase, NodeId, NodeTemplate, Store};
use packed_simd::u8x8;

impl NodeId {
    pub fn expand(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::Interior { nw, ne, sw, se } => {
                let empty = store.create_empty(self.level(store) - 1);

                let nw = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: empty,
                    sw: empty,
                    se: nw,
                });

                let ne = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: empty,
                    sw: ne,
                    se: empty,
                });

                let sw = store.create_interior(NodeTemplate {
                    nw: empty,
                    ne: sw,
                    sw: empty,
                    se: empty,
                });

                let se = store.create_interior(NodeTemplate {
                    nw: se,
                    ne: empty,
                    sw: empty,
                    se: empty,
                });

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
            _ => panic!(),
        }
    }

    pub fn nw(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { board } => {
                let mut board_array = [0; 16];
                board.write_to_slice_unaligned(&mut board_array);
                let level_3_board = u8x8::new(
                    board_array[0].to_be_bytes()[0],
                    board_array[1].to_be_bytes()[0],
                    board_array[2].to_be_bytes()[0],
                    board_array[3].to_be_bytes()[0],
                    board_array[4].to_be_bytes()[0],
                    board_array[5].to_be_bytes()[0],
                    board_array[6].to_be_bytes()[0],
                    board_array[7].to_be_bytes()[0],
                );
                store.create_level_3(level_3_board)
            }
            NodeBase::Interior { nw, .. } => nw,
        }
    }

    pub fn ne(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { board } => {
                let mut board_array = [0; 16];
                board.write_to_slice_unaligned(&mut board_array);
                let level_3_board = u8x8::new(
                    board_array[0].to_be_bytes()[1],
                    board_array[1].to_be_bytes()[1],
                    board_array[2].to_be_bytes()[1],
                    board_array[3].to_be_bytes()[1],
                    board_array[4].to_be_bytes()[1],
                    board_array[5].to_be_bytes()[1],
                    board_array[6].to_be_bytes()[1],
                    board_array[7].to_be_bytes()[1],
                );
                store.create_level_3(level_3_board)
            }
            NodeBase::Interior { ne, .. } => ne,
        }
    }

    pub fn sw(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { board } => {
                let mut board_array = [0; 16];
                board.write_to_slice_unaligned(&mut board_array);
                let level_3_board = u8x8::new(
                    board_array[8].to_be_bytes()[0],
                    board_array[9].to_be_bytes()[0],
                    board_array[10].to_be_bytes()[0],
                    board_array[11].to_be_bytes()[0],
                    board_array[12].to_be_bytes()[0],
                    board_array[13].to_be_bytes()[0],
                    board_array[14].to_be_bytes()[0],
                    board_array[15].to_be_bytes()[0],
                );
                store.create_level_3(level_3_board)
            }
            NodeBase::Interior { sw, .. } => sw,
        }
    }

    pub fn se(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { board } => {
                let mut board_array = [0; 16];
                board.write_to_slice_unaligned(&mut board_array);
                let level_3_board = u8x8::new(
                    board_array[8].to_be_bytes()[1],
                    board_array[9].to_be_bytes()[1],
                    board_array[10].to_be_bytes()[1],
                    board_array[11].to_be_bytes()[1],
                    board_array[12].to_be_bytes()[1],
                    board_array[13].to_be_bytes()[1],
                    board_array[14].to_be_bytes()[1],
                    board_array[15].to_be_bytes()[1],
                );
                store.create_level_3(level_3_board)
            }
            NodeBase::Interior { se, .. } => se,
        }
    }

    pub fn center_subnode(&self, store: &mut Store) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { board } => {
                let mut board_array = [0; 16];
                board.write_to_slice_unaligned(&mut board_array);
                let level_3_board = u8x8::new(
                    util::center(board_array[4]),
                    util::center(board_array[5]),
                    util::center(board_array[6]),
                    util::center(board_array[7]),
                    util::center(board_array[8]),
                    util::center(board_array[9]),
                    util::center(board_array[10]),
                    util::center(board_array[11]),
                );
                store.create_level_3(level_3_board)
            }
            NodeBase::Interior { nw, ne, sw, se } => {
                let template = NodeTemplate {
                    nw: nw.se(store),
                    ne: ne.sw(store),
                    sw: sw.ne(store),
                    se: se.nw(store),
                };
                store.create_interior(template)
            }
        }
    }

    pub fn north_subsubnode(&self, store: &mut Store) -> NodeId {
        let w = self.nw(store);
        let e = self.ne(store);
        centered_horiz(store, w, e)
    }

    pub fn south_subsubnode(&self, store: &mut Store) -> NodeId {
        let w = self.sw(store);
        let e = self.se(store);
        centered_horiz(store, w, e)
    }

    pub fn east_subsubnode(&self, store: &mut Store) -> NodeId {
        let n = self.ne(store);
        let s = self.se(store);
        centered_vert(store, n, s)
    }

    pub fn west_subsubnode(&self, store: &mut Store) -> NodeId {
        let n = self.nw(store);
        let s = self.sw(store);
        centered_vert(store, n, s)
    }
}

fn centered_horiz(store: &mut Store, w: NodeId, e: NodeId) -> NodeId {
    match (store.get(e).base, store.get(w).base) {
        (NodeBase::LevelFour { board: e_board }, NodeBase::LevelFour { board: w_board }) => {
            let mut e_board_array = [0; 16];
            e_board.write_to_slice_unaligned(&mut e_board_array);

            let mut w_board_array = [0; 16];
            w_board.write_to_slice_unaligned(&mut w_board_array);

            let level_3_board = u8x8::new(
                w_board_array[4].to_be_bytes()[1] << 4 | e_board_array[4].to_be_bytes()[0] >> 4,
                w_board_array[5].to_be_bytes()[1] << 4 | e_board_array[5].to_be_bytes()[0] >> 4,
                w_board_array[6].to_be_bytes()[1] << 4 | e_board_array[6].to_be_bytes()[0] >> 4,
                w_board_array[7].to_be_bytes()[1] << 4 | e_board_array[7].to_be_bytes()[0] >> 4,
                w_board_array[8].to_be_bytes()[1] << 4 | e_board_array[8].to_be_bytes()[0] >> 4,
                w_board_array[9].to_be_bytes()[1] << 4 | e_board_array[9].to_be_bytes()[0] >> 4,
                w_board_array[10].to_be_bytes()[1] << 4 | e_board_array[10].to_be_bytes()[0] >> 4,
                w_board_array[11].to_be_bytes()[1] << 4 | e_board_array[11].to_be_bytes()[0] >> 4,
            );
            store.create_level_3(level_3_board)
        }
        (NodeBase::Interior { .. }, NodeBase::Interior { .. }) => {
            let nw = w.ne(store).se(store);
            let ne = e.nw(store).sw(store);
            let sw = w.se(store).ne(store);
            let se = e.sw(store).nw(store);
            store.create_interior(NodeTemplate { nw, ne, sw, se })
        }
        _ => panic!(),
    }
}

fn centered_vert(store: &mut Store, n: NodeId, s: NodeId) -> NodeId {
    match (store.get(n).base, store.get(s).base) {
        (NodeBase::LevelFour { board: n_board }, NodeBase::LevelFour { board: s_board }) => {
            let mut n_board_array = [0; 16];
            n_board.write_to_slice_unaligned(&mut n_board_array);

            let mut s_board_array = [0; 16];
            s_board.write_to_slice_unaligned(&mut s_board_array);

            let level_3_board = u8x8::new(
                util::center(n_board_array[12]),
                util::center(n_board_array[13]),
                util::center(n_board_array[14]),
                util::center(n_board_array[15]),
                util::center(s_board_array[0]),
                util::center(s_board_array[1]),
                util::center(s_board_array[2]),
                util::center(s_board_array[3]),
            );
            store.create_level_3(level_3_board)
        }
        (NodeBase::Interior { .. }, NodeBase::Interior { .. }) => {
            let nw = n.sw(store).se(store);
            let ne = n.se(store).sw(store);
            let sw = s.nw(store).ne(store);
            let se = s.ne(store).nw(store);

            store.create_interior(NodeTemplate { nw, ne, sw, se })
        }
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packed_simd::u16x16;

    fn filled_square(store: &mut Store, level: u8) -> NodeId {
        let mut filled = store.create_empty(level);
        let min = filled.min_coord(store);
        let max = filled.max_coord(store);
        for x in min..=max {
            for y in min..=max {
                filled = filled.set_cell_alive(store, x, y);
            }
        }
        filled
    }

    mod level_4 {
        use super::*;

        #[test]
        fn nw() {
            let mut store = Store::new();
            let node = store.create_level_4(u16x16::new(
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
            ));
            let nw = store.create_level_3(u8x8::splat(0b1111_1111));
            assert_eq!(node.nw(&mut store), nw);
        }

        #[test]
        fn ne() {
            let mut store = Store::new();
            let node = store.create_level_4(u16x16::new(
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
            ));
            let ne = store.create_level_3(u8x8::splat(0b1111_1111));
            assert_eq!(node.ne(&mut store), ne);
        }

        #[test]
        fn sw() {
            let mut store = Store::new();
            let node = store.create_level_4(u16x16::new(
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
                0b1111_1111_0000_0000,
            ));
            let sw = store.create_level_3(u8x8::splat(0b1111_1111));
            assert_eq!(node.sw(&mut store), sw);
        }

        #[test]
        fn se() {
            let mut store = Store::new();
            let node = store.create_level_4(u16x16::new(
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
                0b0000_0000_1111_1111,
            ));
            let se = store.create_level_3(u8x8::splat(0b1111_1111));
            assert_eq!(node.se(&mut store), se);
        }

        #[test]
        fn center_subnode() {
            let mut store = Store::new();
            let node = store.create_level_4(u16x16::new(
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_1111_1111_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
                0b0000_0000_0000_0000,
            ));
            let center_subnode = store.create_level_3(u8x8::splat(0b1111_1111));
            assert_eq!(node.center_subnode(&mut store), center_subnode);
        }
    }

    mod level_5 {
        use super::*;

        #[test]
        fn nw() {
            let mut store = Store::new();
            let empty = store.create_empty(4);
            let filled = filled_square(&mut store, 4);
            let node = store.create_interior(NodeTemplate {
                nw: filled,
                ne: empty,
                sw: empty,
                se: empty,
            });
            assert_eq!(node.nw(&mut store), filled);
        }

        #[test]
        fn ne() {
            let mut store = Store::new();
            let empty = store.create_empty(4);
            let filled = filled_square(&mut store, 4);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: filled,
                sw: empty,
                se: empty,
            });
            assert_eq!(node.ne(&mut store), filled);
        }

        #[test]
        fn sw() {
            let mut store = Store::new();
            let empty = store.create_empty(4);
            let filled = filled_square(&mut store, 4);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: empty,
                sw: filled,
                se: empty,
            });
            assert_eq!(node.sw(&mut store), filled);
        }

        #[test]
        fn se() {
            let mut store = Store::new();
            let empty = store.create_empty(4);
            let filled = filled_square(&mut store, 4);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: empty,
                sw: empty,
                se: filled,
            });
            assert_eq!(node.se(&mut store), filled);
        }

        #[test]
        fn center_subnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(5);
            for x in -8..8 {
                for y in -8..8 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.center_subnode(&mut store),
                filled_square(&mut store, 4)
            );
        }

        #[test]
        fn west_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(5);
            for x in -12..-4 {
                for y in -4..4 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.west_subsubnode(&mut store),
                filled_square(&mut store, 3)
            );
        }

        #[test]
        fn east_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(5);
            for x in 4..12 {
                for y in -4..4 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.east_subsubnode(&mut store),
                filled_square(&mut store, 3)
            );
        }

        #[test]
        fn north_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(5);
            for x in -4..4 {
                for y in -12..-4 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.north_subsubnode(&mut store),
                filled_square(&mut store, 3)
            );
        }

        #[test]
        fn south_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(5);
            for x in -4..4 {
                for y in 4..12 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.south_subsubnode(&mut store),
                filled_square(&mut store, 3)
            );
        }
    }

    mod level_6 {
        use super::*;

        #[test]
        fn nw() {
            let mut store = Store::new();
            let empty = store.create_empty(5);
            let filled = filled_square(&mut store, 5);
            let node = store.create_interior(NodeTemplate {
                nw: filled,
                ne: empty,
                sw: empty,
                se: empty,
            });
            assert_eq!(node.nw(&mut store), filled);
        }

        #[test]
        fn ne() {
            let mut store = Store::new();
            let empty = store.create_empty(5);
            let filled = filled_square(&mut store, 5);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: filled,
                sw: empty,
                se: empty,
            });
            assert_eq!(node.ne(&mut store), filled);
        }

        #[test]
        fn sw() {
            let mut store = Store::new();
            let empty = store.create_empty(5);
            let filled = filled_square(&mut store, 5);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: empty,
                sw: filled,
                se: empty,
            });
            assert_eq!(node.sw(&mut store), filled);
        }

        #[test]
        fn se() {
            let mut store = Store::new();
            let empty = store.create_empty(5);
            let filled = filled_square(&mut store, 5);
            let node = store.create_interior(NodeTemplate {
                nw: empty,
                ne: empty,
                sw: empty,
                se: filled,
            });
            assert_eq!(node.se(&mut store), filled);
        }

        #[test]
        fn center_subnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(6);
            for x in -16..16 {
                for y in -16..16 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.center_subnode(&mut store),
                filled_square(&mut store, 5)
            );
        }

        #[test]
        fn west_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(6);
            for x in -24..-8 {
                for y in -8..8 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.west_subsubnode(&mut store),
                filled_square(&mut store, 4)
            );
        }

        #[test]
        fn east_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(6);
            for x in 8..24 {
                for y in -8..8 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.east_subsubnode(&mut store),
                filled_square(&mut store, 4)
            );
        }

        #[test]
        fn north_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(6);
            for x in -8..8 {
                for y in -24..-8 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.north_subsubnode(&mut store),
                filled_square(&mut store, 4)
            );
        }

        #[test]
        fn south_subsubnode() {
            let mut store = Store::new();
            let mut node = store.create_empty(6);
            for x in -8..8 {
                for y in 8..24 {
                    node = node.set_cell_alive(&mut store, x, y);
                }
            }
            assert_eq!(
                node.south_subsubnode(&mut store),
                filled_square(&mut store, 4)
            );
        }
    }

    #[test]
    fn expand() {
        let mut store = Store::new();
        let center = filled_square(&mut store, 5);
        assert_eq!(center.expand(&mut store).center_subnode(&mut store), center);
    }
}
