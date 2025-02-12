// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use indexmap::IndexSet;
use std::hash::Hash;

pub struct RecentSet<T: Eq + Hash + Clone> {
    capacity: usize,
    set: IndexSet<T>,
}

impl<T: Eq + Hash + Clone> RecentSet<T> {
    pub fn new(capacity: usize) -> Self {
        RecentSet {
            capacity,
            set: IndexSet::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        let ok = self.set.insert(value);
        if self.len() > self.capacity {
            // Create a cloned copy of the first element to avoid the immutable borrow.
            if let Some(first) = self.set.first().cloned() {
                self.set.swap_remove(&first);
            }
        }
        ok
    }

    pub fn contains(&self, value: &T) -> bool {
        self.set.contains(value)
    }

    fn len(&self) -> usize {
        self.set.len()
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_set() {
        let mut set = RecentSet::new(3);
        assert!(set.is_empty());
        assert!(set.insert(1));
        assert!(set.insert(2));
        assert!(set.insert(3));
        assert!(!set.insert(1));
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&4));
        assert_eq!(set.len(), 3);
        assert!(!set.is_empty());
        assert!(set.insert(4));
        assert!(!set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(set.contains(&4));
        assert_eq!(set.len(), 3);
        assert!(!set.is_empty());
    }
}
