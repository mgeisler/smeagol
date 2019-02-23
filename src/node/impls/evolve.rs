/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::node::*;

mod level_5;

fn horiz_jump(store: &mut Store, w: NodeId, e: NodeId) -> NodeId {
    let nw = w.ne(store);
    let ne = e.nw(store);
    let sw = w.se(store);
    let se = e.sw(store);

    store
        .create_interior(NodeTemplate { nw, ne, sw, se })
        .jump(store)
}

fn vert_jump(store: &mut Store, n: NodeId, s: NodeId) -> NodeId {
    let nw = n.sw(store);
    let ne = n.se(store);
    let sw = s.nw(store);
    let se = s.ne(store);

    store
        .create_interior(NodeTemplate { nw, ne, sw, se })
        .jump(store)
}

impl NodeId {
    /// For a level `n` node, advances the node `2^(n-2)` generations into the future.
    ///
    /// Returns a level `n-1` node.
    #[allow(clippy::many_single_char_names)]
    pub fn jump(self, store: &mut Store) -> NodeId {
        if let Some(jump) = store.get_jump(self) {
            return jump;
        }

        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                if level == Level(5) {
                    self::level_5::jump_level_5(store, nw, ne, sw, se)
                } else {
                    let a = nw.jump(store);
                    let b = horiz_jump(store, nw, ne);
                    let c = ne.jump(store);
                    let d = vert_jump(store, nw, sw);
                    let e = self.center_subnode(store).jump(store);
                    let f = vert_jump(store, ne, se);
                    let g = sw.jump(store);
                    let h = horiz_jump(store, sw, se);
                    let i = se.jump(store);

                    let w = store
                        .create_interior(NodeTemplate {
                            nw: a,
                            ne: b,
                            sw: d,
                            se: e,
                        })
                        .jump(store);

                    let x = store
                        .create_interior(NodeTemplate {
                            nw: b,
                            ne: c,
                            sw: e,
                            se: f,
                        })
                        .jump(store);

                    let y = store
                        .create_interior(NodeTemplate {
                            nw: d,
                            ne: e,
                            sw: g,
                            se: h,
                        })
                        .jump(store);

                    let z = store
                        .create_interior(NodeTemplate {
                            nw: e,
                            ne: f,
                            sw: h,
                            se: i,
                        })
                        .jump(store);

                    let jump = store.create_interior(NodeTemplate {
                        nw: w,
                        ne: x,
                        sw: y,
                        se: z,
                    });
                    store.add_jump(self, jump);
                    jump
                }
            }
        }
    }

    /// For a level `n` node, advances the node `step_size` generations into the future.
    ///
    /// The step size is determined by the store.
    ///
    /// Returns a level `n-1` node.
    #[allow(clippy::many_single_char_names)]
    pub fn step(self, store: &mut Store) -> NodeId {
        if let Some(step) = store.get_step(self) {
            return step;
        }

        let step_log_2 = store.step_log_2();

        match store.node(self) {
            Node::Leaf { .. } => panic!(),
            Node::Interior {
                nw,
                ne,
                sw,
                se,
                level,
                ..
            } => {
                if step_log_2 == level.0 - 2 {
                    let step = self.jump(store);
                    store.add_step(self, step);
                    return step;
                }

                if level == Level(5) {
                    let step = self::level_5::step_level_5(store, step_log_2, nw, ne, sw, se);
                    store.add_step(self, step);
                    step
                } else {
                    let a = nw.center_subnode(store);
                    let b = self.north_subsubnode(store);
                    let c = ne.center_subnode(store);
                    let d = self.west_subsubnode(store);
                    let e = self.center_subnode(store).center_subnode(store);
                    let f = self.east_subsubnode(store);
                    let g = sw.center_subnode(store);
                    let h = self.south_subsubnode(store);
                    let i = se.center_subnode(store);

                    let w = store
                        .create_interior(NodeTemplate {
                            nw: a,
                            ne: b,
                            sw: d,
                            se: e,
                        })
                        .step(store);

                    let x = store
                        .create_interior(NodeTemplate {
                            nw: b,
                            ne: c,
                            sw: e,
                            se: f,
                        })
                        .step(store);

                    let y = store
                        .create_interior(NodeTemplate {
                            nw: d,
                            ne: e,
                            sw: g,
                            se: h,
                        })
                        .step(store);

                    let z = store
                        .create_interior(NodeTemplate {
                            nw: e,
                            ne: f,
                            sw: h,
                            se: i,
                        })
                        .step(store);

                    let step = store.create_interior(NodeTemplate {
                        nw: w,
                        ne: x,
                        sw: y,
                        se: z,
                    });
                    store.add_step(self, step);
                    step
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nw_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0111,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: glider,
            ne: empty,
            sw: empty,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_1000_0000,
            0b0000_0000_0100_0000,
            0b0000_0001_1100_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn ne_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0100_0000_0000_0000,
            0b1000_0000_0000_0000,
            0b1110_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: glider,
            sw: empty,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0010_0000_0000,
            0b0000_0011_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn sw_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0111,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: empty,
            sw: glider,
            se: empty,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_1100_0000,
            0b0000_0000_0100_0000,
            0b0000_0000_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn se_glider_jump() {
        let mut store = Store::new();

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b1110_0000_0000_0000,
            0b1000_0000_0000_0000,
            0b0100_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: empty,
            ne: empty,
            sw: empty,
            se: glider,
        });

        let jump = level_5.jump(&mut store);
        let expected_jump = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0011_1000_0000,
            0b0000_0010_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(jump, expected_jump);
    }

    #[test]
    fn nw_glider_step() {
        let mut store = Store::new();
        store.set_step_log_2(2);

        let empty = store.create_empty(Level(4));
        let glider = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0111,
        ));

        let level_5 = store.create_interior(NodeTemplate {
            nw: glider,
            ne: empty,
            sw: empty,
            se: empty,
        });

        let step = level_5.step(&mut store);
        let expected_step = store.create_leaf(u16x16::new(
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0001_0000_0000,
            0b0000_0000_1000_0000,
            0b0000_0011_1000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
            0b0000_0000_0000_0000,
        ));

        assert_eq!(step, expected_step);
    }
}
