use crate::node::{NodeBase, NodeId, Store};

impl NodeId {
    pub fn base(self, store: &Store) -> NodeBase {
        store.get(self).base
    }

    /// Returns the level of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    /// let empty = store.create_empty(7);
    /// assert_eq!(empty.level(&store), 7);
    /// ```
    pub fn level(self, store: &Store) -> u8 {
        store.get(self).level
    }

    /// Returns the number of alive cells in the node.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    ///
    /// let empty = store.create_empty(5);
    /// assert_eq!(empty.population(&store), 0);
    ///
    /// let one_alive = empty.set_cell_alive(&mut store, smeagol::Position::new(0, 0));
    /// assert_eq!(one_alive.population(&store), 1);
    /// ```
    pub fn population(self, store: &Store) -> u128 {
        store.get(self).population
    }

    /// Returns the minimum coordinate that can be used with the node.
    ///
    /// For a level `n` node, this is equal to `-2^n`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    ///
    /// // 32 by 32
    /// let mut stripes = store.create_empty(5);
    ///
    /// // fill with vertical stripes
    /// let min = stripes.min_coord(&store);
    /// let max = stripes.max_coord(&store);
    /// for x in min..=max {
    ///     for y in min..=max {
    ///         let pos = smeagol::Position { x, y };
    ///         stripes = if x % 2 == 0 {
    ///             stripes.set_cell_alive(&mut store, pos)
    ///         } else {
    ///             stripes
    ///         };
    ///     }
    /// }
    ///
    /// // half of the node's cells are alive
    /// assert_eq!(stripes.population(&store), 32 * 32 / 2);
    /// ```
    pub fn min_coord(self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::min_value()
        } else {
            -(1 << (level - 1))
        }
    }

    /// Returns the maximum coordinate that can be used with the node.
    ///
    /// For a level `n` node, this is equal to `2^n - 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = smeagol::node::Store::new();
    ///
    /// // 32 by 32
    /// let mut stripes = store.create_empty(5);
    ///
    /// // fill with horizontal stripes
    /// let min = stripes.min_coord(&store);
    /// let max = stripes.max_coord(&store);
    /// for x in min..=max {
    ///     for y in min..=max {
    ///         let pos = smeagol::Position { x, y };
    ///         stripes = if y % 2 == 0 {
    ///             stripes.set_cell_alive(&mut store, pos)
    ///         } else {
    ///             stripes
    ///         };
    ///     }
    /// }
    ///
    /// // half of the node's cells are alive
    /// assert_eq!(stripes.population(&store), 32 * 32 / 2);
    /// ```
    pub fn max_coord(self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::max_value()
        } else {
            (1 << (level - 1)) - 1
        }
    }
}
