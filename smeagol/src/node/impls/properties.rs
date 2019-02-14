use crate::node::{NodeId, Store};

impl NodeId {
    /// Returns the level of the node.
    pub fn level(&self, store: &Store) -> u8 {
        store.get(*self).level
    }

    /// Returns the population of the node.
    pub fn population(&self, store: &Store) -> u128 {
        store.get(*self).population
    }

    /// Returns the minimum coordinate that can be used with the node.
    pub fn min_coord(&self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::min_value()
        } else {
            -(1 << (level - 1))
        }
    }

    /// Returns the maximum coordinate that can be used with the node.
    pub fn max_coord(&self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::max_value()
        } else {
            (1 << (level - 1)) - 1
        }
    }
}
