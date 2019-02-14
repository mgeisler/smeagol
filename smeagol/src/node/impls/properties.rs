use crate::node::{NodeId, Store};

impl NodeId {
    pub fn level(&self, store: &Store) -> u8 {
        store.get(*self).level
    }

    pub fn population(&self, store: &Store) -> u128 {
        store.get(*self).population
    }

    pub fn min_coord(&self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::min_value()
        } else {
            -(1 << (level - 1))
        }
    }

    pub fn max_coord(&self, store: &Store) -> i64 {
        let level = self.level(store);
        if level == 64 {
            i64::max_value()
        } else {
            (1 << (level - 1)) - 1
        }
    }
}
