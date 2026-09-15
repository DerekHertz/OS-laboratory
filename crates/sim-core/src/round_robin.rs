//! Production round-robin policy for the sched.m1 kernel.
use crate::input::PolicyConfig;
use crate::kernel::Policy;
use std::collections::VecDeque;
use std::num::NonZeroU64;

/// Selects the oldest entry in the global FIFO and grants a fixed useful-time quantum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundRobinPolicy {
    quantum: NonZeroU64,
}

impl RoundRobinPolicy {
    pub const fn new(quantum: NonZeroU64) -> Self {
        Self { quantum }
    }
}

impl Policy for RoundRobinPolicy {
    fn configuration(&self) -> PolicyConfig {
        PolicyConfig::RoundRobin {
            quantum: self.quantum.get(),
        }
    }

    fn quantum(&self) -> Option<NonZeroU64> {
        Some(self.quantum)
    }

    fn select(&mut self, ready: &VecDeque<usize>) -> Option<usize> {
        ready.front().copied()
    }
}

#[cfg(test)]
#[path = "round_robin_tests.rs"]
mod tests;
