//! Purpose:
//! Emits the runtime helpers backing `memory_get_usage()` and `memory_get_peak_usage()`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::system`.
//!
//! Key details:
//! - The numbers are READ, not invented. `__rt_heap_alloc` adds each block's total footprint
//!   (payload + 16-byte header) to `_gc_live` and raises `_gc_peak` on every allocation, and
//!   `__rt_heap_free` subtracts it again; `_heap_off` is the arena bump offset, i.e. every byte
//!   ever carved out of `_heap_buf`. All three are maintained unconditionally — `--heap-debug`
//!   only prints them — so these helpers cost one load and no syscall.
//! - Mode mapping, chosen to match what php's two modes MEAN rather than their magnitudes:
//!   `$real_usage = false` → `_gc_live` / `_gc_peak` (what the allocator has handed out and is
//!   still holding, php's Zend-MM "in use"), `$real_usage = true` → `_heap_off` (what the
//!   allocator has taken from its arena, php's "real size allocated from the system"). A freed
//!   block is parked on the free list rather than returned, so the real figure never decreases,
//!   which is also how php's behaves.
//! - Both helpers take the flag in the INTEGER RESULT register (`x0` / `rax`) and return the
//!   count there too, which is `__rt_hrtime`'s convention and lets the AOT lowering reuse
//!   `resolve_integer_arg_to_result` (the helper that REFUSES an array or a bare string instead
//!   of reading its payload word as a flag). The eval bridge's
//!   `__elephc_eval_memory_get_usage` / `__elephc_eval_memory_get_peak_usage` adapt the C ABI to
//!   it in one instruction, so the interpreter reads the same counters as the compiled program
//!   rather than describing a heap of its own.
//! - Branch-free on purpose: a `.globl` runtime label must never be entered by fall-through
//!   (`-dead_strip` collects unreferenced atoms), and with `csel` / `cmov` there is no internal
//!   label here to collect or to branch to from another atom.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_memory_get_usage`: live heap bytes, or arena bytes taken when the flag is set.
pub fn emit_memory_get_usage(emitter: &mut Emitter) {
    emit_memory_reporter(emitter, "memory_get_usage", "__rt_memory_get_usage", "_gc_live");
}

/// Emits `__rt_memory_get_peak_usage`: the live-byte high watermark, or arena bytes taken.
pub fn emit_memory_get_peak_usage(emitter: &mut Emitter) {
    emit_memory_reporter(
        emitter,
        "memory_get_peak_usage",
        "__rt_memory_get_peak_usage",
        "_gc_peak",
    );
}

/// Emits one reporter selecting between a manager counter and the arena bump offset.
///
/// ## Input register
/// - `x0` / `rax` = `$real_usage` flag (nonzero → arena bytes taken, zero → manager counter)
///
/// ## Output
/// - `x0` / `rax` = the byte count as a plain integer
fn emit_memory_reporter(emitter: &mut Emitter, php_name: &str, label: &str, counter: &str) {
    if emitter.target.arch == Arch::X86_64 {
        emit_memory_reporter_x86_64(emitter, php_name, label, counter);
        return;
    }

    emitter.blank();
    emitter.comment(&format!("--- runtime: {php_name} ---"));
    emitter.label_global(label);
    abi::emit_symbol_address(emitter, "x9", counter);
    emitter.instruction("ldr x9, [x9]");                                        // x9 = manager counter (live or peak bytes)
    abi::emit_symbol_address(emitter, "x10", "_heap_off");
    emitter.instruction("ldr x10, [x10]");                                      // x10 = arena bytes ever carved out of _heap_buf
    emitter.instruction("cmp x0, #0");                                          // was $real_usage truthy?
    emitter.instruction("csel x0, x10, x9, ne");                                // truthy → arena bytes, else the manager counter
    emitter.instruction("ret");                                                 // return the byte count
}

/// Emits the x86_64 variant of one memory reporter.
fn emit_memory_reporter_x86_64(emitter: &mut Emitter, php_name: &str, label: &str, counter: &str) {
    emitter.blank();
    emitter.comment(&format!("--- runtime: {php_name} ---"));
    emitter.label_global(label);
    abi::emit_symbol_address(emitter, "r8", counter);
    emitter.instruction("mov r9, QWORD PTR [r8]");                              // r9 = manager counter (live or peak bytes)
    abi::emit_symbol_address(emitter, "r8", "_heap_off");
    emitter.instruction("mov r10, QWORD PTR [r8]");                             // r10 = arena bytes ever carved out of _heap_buf
    emitter.instruction("test rax, rax");                                       // was $real_usage truthy?
    emitter.instruction("mov rax, r9");                                         // default to the manager counter
    emitter.instruction("cmovne rax, r10");                                     // truthy → arena bytes instead
    emitter.instruction("ret");                                                 // return the byte count
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies each reporter reads ITS OWN counter and that the flag picks the arena offset.
    ///
    /// The assertions name both counters per helper on purpose. A version that read `_gc_live`
    /// for the peak, or that ignored `_heap_off` and answered the manager counter for both
    /// modes, is the kind of mistake that produces a plausible number and no failure anywhere
    /// else -- `memory_get_peak_usage()` would simply track `memory_get_usage()`.
    #[test]
    fn test_memory_reporters_read_their_own_counters_on_aarch64() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_memory_get_usage(&mut emitter);
        emit_memory_get_peak_usage(&mut emitter);
        let asm = emitter.output();

        let usage = asm
            .split("__rt_memory_get_peak_usage:")
            .next()
            .expect("the usage helper is emitted first");
        assert!(usage.contains("_gc_live"), "usage must read the live-byte counter");
        assert!(!usage.contains("_gc_peak"), "usage must not read the peak counter");
        let peak = asm
            .split("__rt_memory_get_peak_usage:")
            .nth(1)
            .expect("the peak helper is emitted second");
        assert!(peak.contains("_gc_peak"), "peak must read the high-watermark counter");
        assert!(!peak.contains("_gc_live"), "peak must not read the live-byte counter");

        // Both select on the flag, and the TRUTHY side is the arena offset, not the counter.
        // One `adrp`, not one mention: `emit_symbol_address` spells the symbol TWICE per use on
        // AArch64 (`adrp …@PAGE` then `add …@PAGEOFF`), so a plain `_heap_off` count reads 4.
        assert_eq!(asm.matches("adrp x10, _heap_off@PAGE\n").count(), 2);
        assert_eq!(asm.matches("csel x0, x10, x9, ne\n").count(), 2);
    }

    /// The x86_64 twin of the same contract, with `cmovne` standing in for `csel`.
    #[test]
    fn test_memory_reporters_read_their_own_counters_on_x86_64() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_memory_get_usage(&mut emitter);
        emit_memory_get_peak_usage(&mut emitter);
        let asm = emitter.output();

        let usage = asm
            .split("__rt_memory_get_peak_usage:")
            .next()
            .expect("the usage helper is emitted first");
        assert!(usage.contains("_gc_live"));
        assert!(!usage.contains("_gc_peak"));
        let peak = asm
            .split("__rt_memory_get_peak_usage:")
            .nth(1)
            .expect("the peak helper is emitted second");
        assert!(peak.contains("_gc_peak"));
        assert!(!peak.contains("_gc_live"));

        assert_eq!(asm.matches("_heap_off").count(), 2);
        assert_eq!(asm.matches("mov rax, r9\n").count(), 2);
        assert_eq!(asm.matches("cmovne rax, r10\n").count(), 2);
    }
}
