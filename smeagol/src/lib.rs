pub mod node;
use self::node::{NodeId, Store};

const INITIAL_LEVEL: u8 = 5;

/// A cell in a Life grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    /// An alive cell.
    Alive,
    /// A dead cell.
    Dead,
}

impl Cell {
    /// Creates a new `Cell`.
    ///
    /// # Examples
    ///
    /// ```
    /// let alive = smeagol::Cell::new(true);
    /// let dead = smeagol::Cell::new(false);
    /// ```
    pub fn new(alive: bool) -> Self {
        if alive {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    /// Returns true for `Cell::Alive` and false for `Cell::Dead`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(smeagol::Cell::Alive.is_alive());
    /// assert!(!smeagol::Cell::Dead.is_alive());
    /// ```
    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}

/// Conway's Game of Life.
#[derive(Clone, Debug)]
pub struct Life {
    root: NodeId,
    store: Store,
    generation: u128,
}

impl Life {
    /// Creates a empty Game of Life.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::new();
    /// ```
    pub fn new() -> Self {
        let mut store = Store::new();
        let root = store.create_empty(INITIAL_LEVEL);
        Self {
            root,
            store,
            generation: 0,
        }
    }

    /// Creates a Game of Life from the given RLE file.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// // pulsar
    /// let mut life = smeagol::Life::from_rle_file("./assets/pulsar.rle")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_rle_file<P>(path: P) -> Result<Self, smeagol_rle::RleError>
    where
        P: AsRef<std::path::Path>,
    {
        let rle = smeagol_rle::Rle::from_file(path)?;
        Ok(Self::from_rle(&rle))
    }

    /// Creates a Game of Life from the given RLE pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// // glider
    /// let mut life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_rle_pattern(pattern: &[u8]) -> Result<Self, smeagol_rle::RleError> {
        let rle = smeagol_rle::Rle::from_pattern(pattern)?;
        Ok(Self::from_rle(&rle))
    }

    fn from_rle(rle: &smeagol_rle::Rle) -> Self {
        let alive_cells = rle
            .alive_cells()
            .into_iter()
            .map(|(x, y)| (i64::from(x), i64::from(y)))
            .collect::<Vec<_>>();

        let mut store = Store::new();
        let mut root = store.create_empty(INITIAL_LEVEL);

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

    /// Returns a `Vec` of the coordinates of the alive cells in the Life grid.
    pub fn get_alive_cells(&self) -> Vec<(i64, i64)> {
        self.root.get_alive_cells(&self.store)
    }

    /// Returns the current generation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut life = smeagol::Life::new();
    /// assert_eq!(life.generation(), 0);
    ///
    /// life.step();
    /// assert_eq!(life.generation(), 1);
    /// ```
    pub fn generation(&self) -> u128 {
        self.generation
    }

    /// Returns the current step size.
    ///
    /// The default step size is 1.
    pub fn step_size(&self) -> u64 {
        1 << self.store.step_log_2()
    }

    /// Sets the step size to be equal to `2^step_log_2`.
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

    /// Advances the Life grid into the future.
    ///
    /// The number of generations advanced is determined by the step size.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), smeagol_rle::RleError> {
    /// // glider
    /// let mut life = smeagol::Life::from_rle_pattern(b"bob$2bo$3o!")?;
    ///
    /// // step size of 1024
    /// life.set_step_log_2(10);
    ///
    /// life.step();
    /// assert_eq!(life.generation(), 1024);
    /// # Ok(())
    /// # }
    /// ```
    pub fn step(&mut self) {
        self.pad();
        self.root = self.root.step(&mut self.store);
        self.generation += u128::from(self.step_size());
    }
}
