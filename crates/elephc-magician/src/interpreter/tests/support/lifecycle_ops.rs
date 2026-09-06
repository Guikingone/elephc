//! Purpose:
//! Release, retain, warning, and echo fake runtime operations.
//!
//! Called from:
//! - `crate::interpreter::tests::support::runtime_ops`.
//!
//! Key details:
//! - These helpers record observable side effects for assertions without touching real runtime memory.

use super::*;

impl FakeOps {
    /// Gives one reference back, recording the release and failing an over-release when counted.
    ///
    /// The handle is not freed, because tests read released cells to assert what they held.
    pub(super) fn runtime_release(&mut self, value: RuntimeCellHandle) -> Result<(), EvalStatus> {
        self.releases.push(value);
        let count_after = self.adjust_refcount(value, -1);
        if count_after < 0 {
            let handle = value.as_ptr() as usize;
            let held = self
                .values
                .get(&handle)
                .map_or_else(|| "<unallocated>".to_string(), |value| format!("{value:?}"));
            self.over_releases.push(FakeOverRelease {
                handle,
                count_after,
                value: held.clone(),
            });
            assert!(
                !self.counting_enforced(),
                "released a fake cell nobody owned: handle {handle} reached count {count_after} \
                 holding {held}. Something gave back a reference it never took -- find the \
                 releasing site in the backtrace below and either stop releasing there or retain \
                 first.\n{}",
                std::backtrace::Backtrace::force_capture()
            );
        }
        Ok(())
    }
    /// Takes one more reference to a fake cell and returns the same handle.
    pub(super) fn runtime_retain(
        &mut self,
        value: RuntimeCellHandle,
    ) -> Result<RuntimeCellHandle, EvalStatus> {
        self.adjust_refcount(value, 1);
        Ok(value)
    }
    /// Records fake PHP warnings without writing to stderr.
    pub(super) fn runtime_warning(&mut self, message: &str) -> Result<(), EvalStatus> {
        self.warnings.push(message.to_string());
        Ok(())
    }
    /// Appends fake echo output for interpreter tests, honoring the fake ob_* stack.
    pub(super) fn runtime_echo(&mut self, value: RuntimeCellHandle) -> Result<(), EvalStatus> {
        let value = self.stringify(value);
        match self.ob_stack.last_mut() {
            Some(level) => level.buffer.push_str(&value),
            None => self.output.push_str(&value),
        }
        Ok(())
    }
}
