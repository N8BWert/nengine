//!
//! Wrapper for Systems to Make them Compatible with the BinaryHeap
//! 

use std::sync::{Arc, RwLock};

pub(crate) struct SystemWrapper<WORLD> {
    pub system: fn(Arc<RwLock<WORLD>>),
    pub update_rate: u128,
    pub priority: u128,
}

impl<WORLD> Ord for SystemWrapper<WORLD> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority).reverse()
    }
}

impl<WORLD> PartialOrd for SystemWrapper<WORLD> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<WORLD> PartialEq for SystemWrapper<WORLD> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<WORLD> Eq for SystemWrapper<WORLD> {}
