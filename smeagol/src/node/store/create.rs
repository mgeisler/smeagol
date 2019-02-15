use crate::node::{Index, Node, NodeBase, NodeId, NodeTemplate, Store};
use packed_simd::{u16x16, u8x8};

impl Store {
    fn add_node(&mut self, node: Node) -> NodeId {
        if let Some(&id) = self.ids.get(&node.base) {
            id
        } else {
            let id = NodeId {
                index: Index(self.nodes.len() as u32),
            };
            self.ids.insert(node.base, id);
            self.nodes.push(node);
            self.steps.push(None);
            self.jumps.push(None);
            id
        }
    }

    pub fn create_level_3(&mut self, board: u8x8) -> NodeId {
        let node = Node {
            base: NodeBase::LevelThree { board },
            level: 3,
            population: board.count_ones().wrapping_sum() as u128,
        };
        self.add_node(node)
    }

    pub fn create_level_4(&mut self, board: u16x16) -> NodeId {
        let node = Node {
            base: NodeBase::LevelFour { board },
            level: 4,
            population: board.count_ones().wrapping_sum() as u128,
        };
        self.add_node(node)
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> NodeId {
        let level = self.get(template.nw).level;
        let new_level = level + 1;
        if new_level > 64 {
            panic!();
        }

        match level {
            3 => {
                match (
                    self.get(template.nw).base,
                    self.get(template.ne).base,
                    self.get(template.sw).base,
                    self.get(template.se).base,
                ) {
                    (
                        NodeBase::LevelThree { board: nw_board },
                        NodeBase::LevelThree { board: ne_board },
                        NodeBase::LevelThree { board: sw_board },
                        NodeBase::LevelThree { board: se_board },
                    ) => {
                        let mut nw_board_array = [0; 8];
                        nw_board.write_to_slice_unaligned(&mut nw_board_array);

                        let mut ne_board_array = [0; 8];
                        ne_board.write_to_slice_unaligned(&mut ne_board_array);

                        let mut sw_board_array = [0; 8];
                        sw_board.write_to_slice_unaligned(&mut sw_board_array);

                        let mut se_board_array = [0; 8];
                        se_board.write_to_slice_unaligned(&mut se_board_array);

                        let board = u16x16::new(
                            u16::from_be_bytes([nw_board_array[0], ne_board_array[0]]),
                            u16::from_be_bytes([nw_board_array[1], ne_board_array[1]]),
                            u16::from_be_bytes([nw_board_array[2], ne_board_array[2]]),
                            u16::from_be_bytes([nw_board_array[3], ne_board_array[3]]),
                            u16::from_be_bytes([nw_board_array[4], ne_board_array[4]]),
                            u16::from_be_bytes([nw_board_array[5], ne_board_array[5]]),
                            u16::from_be_bytes([nw_board_array[6], ne_board_array[6]]),
                            u16::from_be_bytes([nw_board_array[7], ne_board_array[7]]),
                            u16::from_be_bytes([sw_board_array[0], se_board_array[0]]),
                            u16::from_be_bytes([sw_board_array[1], se_board_array[1]]),
                            u16::from_be_bytes([sw_board_array[2], se_board_array[2]]),
                            u16::from_be_bytes([sw_board_array[3], se_board_array[3]]),
                            u16::from_be_bytes([sw_board_array[4], se_board_array[4]]),
                            u16::from_be_bytes([sw_board_array[5], se_board_array[5]]),
                            u16::from_be_bytes([sw_board_array[6], se_board_array[6]]),
                            u16::from_be_bytes([sw_board_array[7], se_board_array[7]]),
                        );

                        self.create_level_4(board)
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let node = Node {
                    base: NodeBase::Interior {
                        nw: template.nw,
                        ne: template.ne,
                        sw: template.sw,
                        se: template.se,
                    },
                    level: new_level,
                    population: self.get(template.nw).population
                        + self.get(template.ne).population
                        + self.get(template.sw).population
                        + self.get(template.se).population,
                };
                self.add_node(node)
            }
        }
    }

    pub fn create_empty(&mut self, level: u8) -> NodeId {
        match level {
            0 | 1 | 2 => panic!(),
            3 => self.create_level_3(u8x8::splat(0)),
            4 => self.create_level_4(u16x16::splat(0)),
            _ => {
                let empty = self.create_empty(level - 1);
                self.create_interior(NodeTemplate {
                    nw: empty,
                    ne: empty,
                    sw: empty,
                    se: empty,
                })
            }
        }
    }
}
