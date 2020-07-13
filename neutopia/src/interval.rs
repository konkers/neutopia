//! A data structure for accounting intervals.
//!
//! This is implemented with a brute force approach that traverses every
//! interval on each add.  A better approach would be to use an interval
//! tree.

use std::cmp::{max, min};
use std::fmt::Debug;

/// An interval from [`start`, `end`)
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Interval<T: Ord + Copy + Debug> {
    /// The start of the interval (inclusive).
    pub start: T,

    /// The end of the interval (exclusive).
    pub end: T,
}

impl<T: Ord + Copy + Debug> Interval<T> {
    /// Returns true if `self` and `other` can be combined.
    ///
    /// This is different that testing for overlapping in that two intervals
    /// that are adjacent are allowed to merge.
    pub fn can_merge(&self, other: &Self) -> bool {
        (self.start <= other.start && other.start <= self.end)
            || (other.start <= self.start && self.start <= other.end)
    }

    /// Merge `other` into this interval
    ///
    /// Panics if the intervals can't merge.
    pub fn merge(&mut self, other: &Self) {
        assert!(self.can_merge(other));
        self.start = min(self.start, other.start);
        self.end = max(self.end, other.end);
    }
}

#[derive(Debug)]
pub struct IntervalStore<T: Ord + Copy + Debug> {
    intervals: Vec<Interval<T>>,
}

impl<T: Ord + Copy + Debug> Default for IntervalStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Copy + Debug> IntervalStore<T> {
    /// Generate a new empty IntervalStore.
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    /// Add an interval to the store.
    pub fn add(&mut self, start: T, end: T) {
        let mut new_interval = Interval { start, end };
        let mut first_match = None;
        let mut i = 0;
        while i != self.intervals.len() {
            let interval = self.intervals[i];
            if first_match.is_none() && interval.can_merge(&new_interval) {
                self.intervals[i].merge(&new_interval);
                new_interval = self.intervals[i];
                first_match = Some(i);
                i += 1;
            } else if first_match.is_some() && interval.can_merge(&new_interval) {
                let match_idx = first_match.unwrap();
                self.intervals[match_idx].merge(&interval);
                self.intervals.remove(i);
            } else {
                i += 1;
            }
        }
        if first_match.is_none() {
            self.intervals.push(new_interval)
        }
    }

    /// Return a owned, sorted Vec of intervals in the store.
    pub fn get_intervals(&self) -> Vec<Interval<T>> {
        let mut intervals = self.intervals.clone();
        intervals.sort();
        intervals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn no_overlap() {
        let mut store = IntervalStore::new();
        store.add(0u32, 2);
        store.add(3, 5);
        store.add(6, 8);
        let mut intervals = store.intervals;
        intervals.sort();
        assert_eq!(
            intervals,
            vec![
                Interval { start: 0, end: 2 },
                Interval { start: 3, end: 5 },
                Interval { start: 6, end: 8 },
            ]
        );
    }

    #[test]
    pub fn adjacent_overlap() {
        let mut store = IntervalStore::new();
        store.add(0u32, 2);
        store.add(4, 6);
        store.add(2, 4);
        let mut intervals = store.intervals;
        intervals.sort();
        assert_eq!(intervals, vec![Interval { start: 0, end: 6 }]);
    }

    #[test]
    pub fn full_overlap() {
        let mut store = IntervalStore::new();
        store.add(0u32, 2);
        store.add(4, 6);
        store.add(1, 5);
        let mut intervals = store.intervals;
        intervals.sort();
        assert_eq!(intervals, vec![Interval { start: 0, end: 6 }]);
    }
}
