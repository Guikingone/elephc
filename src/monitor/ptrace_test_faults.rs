//! Purpose:
//! Stages delayed ptrace stops without requiring an uninterruptible kernel task.
//!
//! Called from:
//! - Linux ptrace process tests and live caller regressions.
//!
//! Key details:
//! - Only interrupt delivery is suppressed; seize, waitpid and detach stay real.
//! - Thread-local state isolates concurrently running tests and is reset on drop.

use std::cell::Cell;

thread_local! {
    static FAULT: Cell<(u32, usize, usize)> = const { Cell::new((0, 0, 0)) };
}

pub(crate) struct DelayedStops;

impl DelayedStops {
    /// Suppresses the next `count` interrupts for one live fixture tid.
    pub(crate) fn new(tid: u32, count: usize) -> Self {
        FAULT.with(|state| {
            assert_eq!(state.get().0, 0, "nested ptrace fault fixture");
            state.set((tid, count, 0));
        });
        Self
    }

    /// Counts actual detach requests, including invalid attempts on running tasks.
    pub(crate) fn detach_attempts(&self) -> usize {
        FAULT.with(|state| state.get().2)
    }
}

impl Drop for DelayedStops {
    /// Restores ordinary interrupt delivery even when a test assertion fails.
    fn drop(&mut self) {
        FAULT.with(|state| state.set((0, 0, 0)));
    }
}

/// Pretends an interrupt was delivered while leaving the seized fixture running.
pub(super) fn withhold_interrupt(tid: u32) -> bool {
    FAULT.with(|state| {
        let (target, remaining, detaches) = state.get();
        if target != tid || remaining == 0 {
            return false;
        }
        state.set((target, remaining - 1, detaches));
        true
    })
}

/// Observes the production detach syscall boundary for the active fixture.
pub(super) fn record_detach(tid: u32) {
    FAULT.with(|state| {
        let (target, remaining, detaches) = state.get();
        if target == tid {
            state.set((target, remaining, detaches + 1));
        }
    });
}
