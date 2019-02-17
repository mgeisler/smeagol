use crate::{
    node::{NodeBase, NodeId, NodeTemplate, Store},
    Cell, Position, Quadrant,
};

const MIN_LVL3_COORD: i64 = -4;
const MAX_LVL3_COORD: i64 = 3;
const MIN_LVL4_COORD: i64 = -8;
const MAX_LVL4_COORD: i64 = 7;

impl NodeId {
    pub fn get_cell(&self, store: &Store, pos: Position) -> Cell {
        match self.base(store) {
            NodeBase::LevelThree { board } => {
                let x_offset = (3 - pos.x) as usize;
                let y_offset = (pos.y + 4) as usize;
                Cell::new(board.extract(y_offset) & (1 << x_offset) > 0)
            }
            NodeBase::LevelFour { board } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                Cell::new(board.extract(y_offset) & (1 << x_offset) > 0)
            }
            NodeBase::Interior { ne, nw, se, sw } => {
                // quarter side length
                let offset = 1 << (self.level(store) - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => nw.get_cell(store, pos.offset(offset, offset)),
                    Quadrant::Northeast => ne.get_cell(store, pos.offset(-offset, offset)),
                    Quadrant::Southwest => sw.get_cell(store, pos.offset(offset, -offset)),
                    Quadrant::Southeast => se.get_cell(store, pos.offset(-offset, -offset)),
                }
            }
        }
    }

    pub fn set_cell_alive(&self, store: &mut Store, pos: Position) -> NodeId {
        match self.base(store) {
            NodeBase::LevelThree { board } => {
                let x_offset = (3 - pos.x) as usize;
                let y_offset = (pos.y + 4) as usize;
                let board = board.replace(y_offset, board.extract(y_offset) | (1 << x_offset));
                store.create_level_3(board)
            }
            NodeBase::LevelFour { board } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                let board = board.replace(y_offset, board.extract(y_offset) | (1 << x_offset));
                store.create_level_4(board)
            }
            NodeBase::Interior { ne, nw, se, sw } => {
                // quarter side length
                let offset = 1 << (self.level(store) - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => {
                        let nw = nw.set_cell_alive(store, pos.offset(offset, offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Northeast => {
                        let ne = ne.set_cell_alive(store, pos.offset(-offset, offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Southwest => {
                        let sw = sw.set_cell_alive(store, pos.offset(offset, -offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    Quadrant::Southeast => {
                        let se = se.set_cell_alive(store, pos.offset(-offset, -offset));
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                }
            }
        }
    }

    pub fn get_alive_cells(&self, store: &Store) -> Vec<Position> {
        match self.base(store) {
            NodeBase::LevelThree { .. } => {
                let mut alive_coords = Vec::with_capacity(64);
                for x in MIN_LVL3_COORD..=MAX_LVL3_COORD {
                    for y in MIN_LVL3_COORD..=MAX_LVL3_COORD {
                        let pos = Position { x, y };
                        if self.get_cell(store, pos).is_alive() {
                            alive_coords.push(pos);
                        }
                    }
                }
                alive_coords
            }
            NodeBase::LevelFour { .. } => {
                let mut alive_coords = Vec::with_capacity(64);
                for x in MIN_LVL4_COORD..=MAX_LVL4_COORD {
                    for y in MIN_LVL4_COORD..=MAX_LVL4_COORD {
                        let pos = Position { x, y };
                        if self.get_cell(store, pos).is_alive() {
                            alive_coords.push(pos);
                        }
                    }
                }
                alive_coords
            }
            NodeBase::Interior { nw, ne, sw, se } => {
                let pop = self.population(store);
                let mut alive_cells = Vec::with_capacity(pop as usize);

                if pop > 0 {
                    // quarter side length
                    let offset = 1 << (self.level(store) - 2);

                    alive_cells.extend(
                        nw.get_alive_cells(store)
                            .into_iter()
                            .map(|pos| pos.offset(-offset, -offset)),
                    );
                    alive_cells.extend(
                        ne.get_alive_cells(store)
                            .into_iter()
                            .map(|pos| pos.offset(offset, -offset)),
                    );
                    alive_cells.extend(
                        sw.get_alive_cells(store)
                            .into_iter()
                            .map(|pos| pos.offset(-offset, offset)),
                    );
                    alive_cells.extend(
                        se.get_alive_cells(store)
                            .into_iter()
                            .map(|pos| pos.offset(offset, offset)),
                    );
                }

                alive_cells
            }
        }
    }

    pub fn set_cells_alive(
        &self,
        store: &mut Store,
        coords: impl IntoIterator<Item = Position>,
    ) -> NodeId {
        self.set_cells_alive_recursive(store, &mut coords.into_iter().collect::<Vec<_>>(), 0, 0)
    }

    fn set_cells_alive_recursive(
        &self,
        store: &mut Store,
        coords: &mut [Position],
        offset_x: i64,
        offset_y: i64,
    ) -> NodeId {
        if coords.is_empty() {
            return *self;
        }

        match self.base(store) {
            NodeBase::LevelThree { mut board } => {
                for &mut pos in coords {
                    let x = (3 - (pos.x - offset_x)) as usize;
                    let y = ((pos.y - offset_y) + 4) as usize;
                    board = board.replace(y, board.extract(y) | (1 << x));
                }
                store.create_level_3(board)
            }
            NodeBase::LevelFour { mut board } => {
                for &mut pos in coords {
                    let x = (7 - (pos.x - offset_x)) as usize;
                    let y = ((pos.y - offset_y) + 8) as usize;
                    board = board.replace(y, board.extract(y) | (1 << x));
                }
                store.create_level_4(board)
            }
            NodeBase::Interior { nw, ne, sw, se } => {
                let vert_cutoff = partition_vert(coords, offset_y);
                let (north, south) = coords.split_at_mut(vert_cutoff);

                let horiz_cutoff = partition_horiz(north, offset_x);
                let (northwest, northeast) = north.split_at_mut(horiz_cutoff);

                let horiz_cutoff = partition_horiz(south, offset_x);
                let (southwest, southeast) = south.split_at_mut(horiz_cutoff);

                // quarter side length
                let offset = 1 << (self.level(store) - 2);

                let nw = nw.set_cells_alive_recursive(
                    store,
                    northwest,
                    offset_x - offset,
                    offset_y - offset,
                );
                let ne = ne.set_cells_alive_recursive(
                    store,
                    northeast,
                    offset_x + offset,
                    offset_y - offset,
                );
                let sw = sw.set_cells_alive_recursive(
                    store,
                    southwest,
                    offset_x - offset,
                    offset_y + offset,
                );
                let se = se.set_cells_alive_recursive(
                    store,
                    southeast,
                    offset_x + offset,
                    offset_y + offset,
                );

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
    }

    pub fn contains_alive_cells(
        &self,
        store: &Store,
        upper_left: Position,
        lower_right: Position,
    ) -> bool {
        assert!(upper_left.x <= lower_right.x);
        assert!(upper_left.y <= lower_right.y);

        // quarter side length
        let offset = 1 << (self.level(store) - 2);

        if self.population(store) == 0 {
            false
        } else {
            match self.base(store) {
                NodeBase::LevelThree { .. } | NodeBase::LevelFour { .. } => {
                    for x in upper_left.x..=lower_right.x {
                        for y in upper_left.y..=lower_right.y {
                            let pos = Position { x, y };
                            if self.get_cell(store, pos).is_alive() {
                                return true;
                            }
                        }
                    }
                    false
                }
                NodeBase::Interior { nw, ne, sw, se } => {
                    match (upper_left.quadrant(), lower_right.quadrant()) {
                        (Quadrant::Northwest, Quadrant::Northwest) => nw.contains_alive_cells(
                            store,
                            upper_left.offset(offset, offset),
                            lower_right.offset(offset, offset),
                        ),
                        (Quadrant::Northeast, Quadrant::Northeast) => ne.contains_alive_cells(
                            store,
                            upper_left.offset(-offset, offset),
                            lower_right.offset(-offset, offset),
                        ),
                        (Quadrant::Southwest, Quadrant::Southwest) => sw.contains_alive_cells(
                            store,
                            upper_left.offset(offset, -offset),
                            lower_right.offset(offset, -offset),
                        ),
                        (Quadrant::Southeast, Quadrant::Southeast) => se.contains_alive_cells(
                            store,
                            upper_left.offset(-offset, -offset),
                            lower_right.offset(-offset, -offset),
                        ),

                        (Quadrant::Northwest, Quadrant::Northeast) => {
                            let nw_lower_right = Position::new(-1, lower_right.y);
                            let ne_upper_left = Position::new(0, upper_left.y);
                            nw.contains_alive_cells(
                                store,
                                upper_left.offset(offset, offset),
                                nw_lower_right.offset(offset, offset),
                            ) || ne.contains_alive_cells(
                                store,
                                ne_upper_left.offset(-offset, offset),
                                lower_right.offset(-offset, offset),
                            )
                        }
                        (Quadrant::Northwest, Quadrant::Southwest) => {
                            let nw_lower_right = Position::new(lower_right.x, -1);
                            let sw_upper_left = Position::new(upper_left.x, 0);
                            nw.contains_alive_cells(
                                store,
                                upper_left.offset(offset, offset),
                                nw_lower_right.offset(offset, offset),
                            ) || sw.contains_alive_cells(
                                store,
                                sw_upper_left.offset(offset, -offset),
                                lower_right.offset(offset, -offset),
                            )
                        }
                        (Quadrant::Southwest, Quadrant::Southeast) => {
                            let sw_lower_right = Position::new(-1, lower_right.y);
                            let se_upper_left = Position::new(0, upper_left.y);
                            sw.contains_alive_cells(
                                store,
                                upper_left.offset(offset, -offset),
                                sw_lower_right.offset(offset, -offset),
                            ) || ne.contains_alive_cells(
                                store,
                                se_upper_left.offset(-offset, -offset),
                                lower_right.offset(-offset, -offset),
                            )
                        }
                        (Quadrant::Northeast, Quadrant::Southeast) => {
                            let ne_lower_right = Position::new(lower_right.x, -1);
                            let se_upper_left = Position::new(upper_left.x, 0);
                            ne.contains_alive_cells(
                                store,
                                upper_left.offset(-offset, offset),
                                ne_lower_right.offset(-offset, offset),
                            ) || se.contains_alive_cells(
                                store,
                                se_upper_left.offset(-offset, -offset),
                                lower_right.offset(-offset, -offset),
                            )
                        }

                        (Quadrant::Northwest, Quadrant::Southeast) => {
                            let nw_upper_left = upper_left;
                            let nw_lower_right = Position::new(-1, -1);

                            let ne_upper_left = Position::new(0, upper_left.y);
                            let ne_lower_right = Position::new(lower_right.x, -1);

                            let sw_upper_left = Position::new(upper_left.x, 0);
                            let sw_lower_right = Position::new(-1, lower_right.y);

                            let se_upper_left = Position::new(0, 0);
                            let se_lower_right = lower_right;

                            nw.contains_alive_cells(
                                store,
                                nw_upper_left.offset(offset, offset),
                                nw_lower_right.offset(offset, offset),
                            ) || ne.contains_alive_cells(
                                store,
                                ne_upper_left.offset(-offset, offset),
                                ne_lower_right.offset(-offset, offset),
                            ) || sw.contains_alive_cells(
                                store,
                                sw_upper_left.offset(offset, -offset),
                                sw_lower_right.offset(offset, -offset),
                            ) || se.contains_alive_cells(
                                store,
                                se_upper_left.offset(-offset, -offset),
                                se_lower_right.offset(-offset, -offset),
                            )
                        }

                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

fn partition_horiz(coords: &mut [Position], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].x < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

fn partition_vert(coords: &mut [Position], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].y < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_set_helper(level: u8) {
        let mut store = Store::new();
        let empty = store.create_empty(level);

        let min = empty.min_coord(&store);
        let max = empty.max_coord(&store);
        for x in min..=max {
            for y in min..=max {
                let pos = Position { x, y };
                let one_alive = empty.set_cell_alive(&mut store, pos);
                let also_one_alive = empty.set_cells_alive(&mut store, vec![pos]);
                assert_eq!(one_alive, also_one_alive);
                assert!(one_alive.get_cell(&store, pos).is_alive());
                assert_eq!(one_alive.get_alive_cells(&store), vec![pos]);
                assert_eq!(one_alive.population(&store), 1);
                assert!(one_alive.contains_alive_cells(&store, pos, pos));
                assert!(one_alive.contains_alive_cells(
                    &store,
                    Position::new(min, min),
                    Position::new(max, max)
                ));
            }
        }
    }

    mod level_3 {
        use super::*;

        #[test]
        fn get_set() {
            get_set_helper(3);
        }
    }

    mod level_4 {
        use super::*;

        #[test]
        fn get_set() {
            get_set_helper(4);
        }
    }

    mod level_5 {
        use super::*;

        #[test]
        fn get_set() {
            get_set_helper(5);
        }
    }
}
