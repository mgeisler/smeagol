use crate::{
    node::{NodeBase, NodeId, NodeTemplate, Store},
    Cell,
};

impl NodeId {
    pub fn get_cell(&self, store: &Store, x: i64, y: i64) -> Cell {
        match store.get(*self).base {
            NodeBase::LevelThree { board } => {
                let x_offset = (3 - x) as usize;
                let y_offset = (y + 4) as usize;
                Cell::new(board.extract(y_offset) & (1 << x_offset) > 0)
            }
            NodeBase::LevelFour { board } => {
                let x_offset = (7 - x) as usize;
                let y_offset = (y + 8) as usize;
                Cell::new(board.extract(y_offset) & (1 << x_offset) > 0)
            }
            NodeBase::Interior { ne, nw, se, sw } => {
                let offset = 1 << (self.level(store) - 2);

                match (x < 0, y < 0) {
                    (true, true) => {
                        // nw
                        nw.get_cell(store, x + offset, y + offset)
                    }
                    (false, true) => {
                        // ne
                        ne.get_cell(store, x - offset, y + offset)
                    }
                    (true, false) => {
                        // sw
                        sw.get_cell(store, x + offset, y - offset)
                    }
                    (false, false) => {
                        // se
                        se.get_cell(store, x - offset, y - offset)
                    }
                }
            }
        }
    }

    pub fn set_cell_alive(&self, store: &mut Store, x: i64, y: i64) -> NodeId {
        match store.get(*self).base {
            NodeBase::LevelThree { board } => {
                let x_offset = (3 - x) as usize;
                let y_offset = (y + 4) as usize;
                let board = board.replace(y_offset, board.extract(y_offset) | (1 << x_offset));
                store.create_level_3(board)
            }
            NodeBase::LevelFour { board } => {
                let x_offset = (7 - x) as usize;
                let y_offset = (y + 8) as usize;
                let board = board.replace(y_offset, board.extract(y_offset) | (1 << x_offset));
                store.create_level_4(board)
            }
            NodeBase::Interior { ne, nw, se, sw } => {
                let offset = 1 << (self.level(store) - 2);

                match (x < 0, y < 0) {
                    (true, true) => {
                        // nw
                        let nw = nw.set_cell_alive(store, x + offset, y + offset);
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    (false, true) => {
                        // ne
                        let ne = ne.set_cell_alive(store, x - offset, y + offset);
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    (true, false) => {
                        // sw
                        let sw = sw.set_cell_alive(store, x + offset, y - offset);
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                    (false, false) => {
                        // se
                        let se = se.set_cell_alive(store, x - offset, y - offset);
                        store.create_interior(NodeTemplate { nw, ne, sw, se })
                    }
                }
            }
        }
    }

    pub fn get_alive_cells(&self, store: &Store) -> Vec<(i64, i64)> {
        match store.get(*self).base {
            NodeBase::LevelThree { .. } => panic!(),
            NodeBase::LevelFour { .. } => {
                let mut alive_coords = Vec::with_capacity(64);
                for x in -8..8 {
                    for y in -8..8 {
                        if self.get_cell(store, x, y).is_alive() {
                            alive_coords.push((x, y));
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
                            .map(|(x, y)| (x - offset, y - offset)),
                    );
                    alive_cells.extend(
                        ne.get_alive_cells(store)
                            .into_iter()
                            .map(|(x, y)| (x + offset, y - offset)),
                    );
                    alive_cells.extend(
                        sw.get_alive_cells(store)
                            .into_iter()
                            .map(|(x, y)| (x - offset, y + offset)),
                    );
                    alive_cells.extend(
                        se.get_alive_cells(store)
                            .into_iter()
                            .map(|(x, y)| (x + offset, y + offset)),
                    );
                }

                alive_cells
            }
        }
    }

    pub fn set_cells_alive(
        &self,
        store: &mut Store,
        coords: impl IntoIterator<Item = (i64, i64)>,
    ) -> NodeId {
        self.set_cells_alive_recursive(store, &mut coords.into_iter().collect::<Vec<_>>(), 0, 0)
    }

    fn set_cells_alive_recursive(
        &self,
        store: &mut Store,
        coords: &mut [(i64, i64)],
        offset_x: i64,
        offset_y: i64,
    ) -> NodeId {
        if coords.is_empty() {
            return *self;
        }

        match store.get(*self).base {
            NodeBase::LevelFour { mut board } => {
                for &mut (x, y) in coords {
                    let x = (7 - (x - offset_x)) as usize;
                    let y = ((y - offset_y) + 8) as usize;
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
            _ => panic!(),
        }
    }
}

fn partition_horiz(coords: &mut [(i64, i64)], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].0 < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}

fn partition_vert(coords: &mut [(i64, i64)], pivot: i64) -> usize {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].1 < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    next_index
}
