//! Production first-come, first-served policy for the sched.m1 kernel.
use crate::input::PolicyConfig;
use crate::kernel::Policy;
use std::collections::VecDeque;
use std::num::NonZeroU64;

/// Selects the oldest entry in the kernel's single global FIFO ready queue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FcfsPolicy;

impl Policy for FcfsPolicy {
    fn configuration(&self) -> PolicyConfig {
        PolicyConfig::Fcfs
    }

    fn quantum(&self) -> Option<NonZeroU64> {
        None
    }

    fn select(&mut self, ready: &VecDeque<usize>) -> Option<usize> {
        ready.front().copied()
    }
}

#[cfg(test)]
#[path = "fcfs_tests.rs"]
mod tests;
