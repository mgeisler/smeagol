/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::*;

const INITIAL_LEVEL: u8 = 7;

#[derive(Clone)]
pub struct Life {
    root: Rc<Node>,
    store: Store,
    generation: u128,
}

impl Life {
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(INITIAL_LEVEL);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn from_rle_file<P>(path: P) -> Result<Self, failure::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = Rle::from_file(path)?;
        Ok(Self::from_rle(&rle))
    }

    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, failure::Error> {
        let rle = Rle::from_pattern(pattern)?;
        Ok(Self::from_rle(&rle))
    }

    pub fn from_rle(rle: &Rle) -> Self {
        let alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| Position::new(i64::from(x), i64::from(y)))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(INITIAL_LEVEL);

        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().min_by_key(|pos| pos.x).unwrap().x;
            let x_max = alive_cells.iter().max_by_key(|pos| pos.x).unwrap().x;
            let y_min = alive_cells.iter().min_by_key(|pos| pos.y).unwrap().y;
            let y_max = alive_cells.iter().max_by_key(|pos| pos.y).unwrap().y;

            while x_min < root.min_coord()
                || x_max > root.max_coord()
                || y_min < root.min_coord()
                || y_max > root.max_coord()
            {
                root = root.expand(&mut store);
            }

            root = root.set_cells_alive(&mut store, alive_cells);
        }

        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn set_cell_alive(&mut self, position: Position) {
        while position.x < self.root.min_coord()
            || position.y < self.root.min_coord()
            || position.x > self.root.max_coord()
            || position.y > self.root.max_coord()
        {
            self.root = self.root.expand(&mut self.store);
        }
        self.root = self.root.set_cell_alive(&mut self.store, position);
    }

    pub fn get_alive_cells(&self) -> Vec<Position> {
        self.root.get_alive_cells()
    }

    pub fn bounding_box(&self) -> Option<BoundingBox> {
        self.root.bounding_box()
    }

    pub fn generation(&self) -> u128 {
        self.generation
    }

    pub fn population(&self) -> u128 {
        self.root.population
    }

    pub fn step_size(&self) -> u64 {
        1 << self.store.step_log_2()
    }

    pub fn step_log_2(&self) -> u8 {
        self.store.step_log_2()
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        self.store.set_step_log_2(step_log_2);
    }

    fn pad(&mut self) {
        while self.root.level < INITIAL_LEVEL
            || self.store.step_log_2() > self.root.level - 2
            || Node::ne(&self.root).population != Node::ne(&self.root).sw().sw().population
            || self.root.nw().population != self.root.nw().se().se().population
            || self.root.se().population != self.root.se().nw().nw().population
            || self.root.sw().population != Node::ne(&Node::ne(&self.root.sw())).population
        {
            self.root = self.root.expand(&mut self.store);
        }
    }

    pub fn step(&mut self) {
        self.pad();
        self.root = self.root.step(&mut self.store);
        self.generation += u128::from(self.step_size());
    }
}

impl Default for Life {
    fn default() -> Self {
        Self::new()
    }
}
