pub mod node;
use self::node::{NodeId, Store};

#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    pub fn new(alive: bool) -> Self {
        if alive {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Life {
    root: NodeId,
    store: Store,
    generation: u128,
}

impl Life {
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(5);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    pub fn from_rle(rle: &smeagol_rle::Rle) -> Self {
        let alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| (i64::from(x), i64::from(y)))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(5);

        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().min_by_key(|&(x, _)| x).unwrap().0;
            let x_max = alive_cells.iter().max_by_key(|&(x, _)| x).unwrap().0;
            let y_min = alive_cells.iter().min_by_key(|&(_, y)| y).unwrap().1;
            let y_max = alive_cells.iter().max_by_key(|&(_, y)| y).unwrap().1;

            while x_min < root.min_coord(&store)
                || x_max > root.max_coord(&store)
                || y_min < root.min_coord(&store)
                || y_max > root.max_coord(&store)
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

    pub fn get_alive_cells(&self) -> Vec<(i64, i64)> {
        self.root.get_alive_cells(&self.store)
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        self.store.set_step_log_2(step_log_2);
    }

    fn pad(&mut self) {
        while self.root.level(&self.store) < 6
            || self.store.step_log_2() > self.root.level(&self.store) - 2
            || self.root.ne(&mut self.store).population(&mut self.store)
                != self
                    .root
                    .ne(&mut self.store)
                    .sw(&mut self.store)
                    .sw(&mut self.store)
                    .population(&mut self.store)
            || self.root.nw(&mut self.store).population(&mut self.store)
                != self
                    .root
                    .nw(&mut self.store)
                    .se(&mut self.store)
                    .se(&mut self.store)
                    .population(&mut self.store)
            || self.root.se(&mut self.store).population(&mut self.store)
                != self
                    .root
                    .se(&mut self.store)
                    .nw(&mut self.store)
                    .nw(&mut self.store)
                    .population(&mut self.store)
            || self.root.sw(&mut self.store).population(&mut self.store)
                != self
                    .root
                    .sw(&mut self.store)
                    .ne(&mut self.store)
                    .ne(&mut self.store)
                    .population(&mut self.store)
        {
            self.root = self.root.expand(&mut self.store);
        }
    }

    pub fn step(&mut self) {
        self.pad();
        self.root = self.root.step(&mut self.store);
    }
}
