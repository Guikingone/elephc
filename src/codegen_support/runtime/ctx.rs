//! Purpose:
//! Owns the per-context runtime state layout (`_rt_ctx`) for the sandbox-threads spike.
//! Defines the ctx-register convention, emits the ctx data block, and emits the
//! `__rt_ctx_init` helper that publishes the ctx pointer into the reserved register.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` (helper emission).
//! - `crate::codegen_support::runtime::data::fixed` (ctx data block emission).
//! - `src/codegen/frame.rs` main prologue (ctx register publication).
//!
//! Key details:
//! - The ctx register is x28 on AArch64 and r14 on x86_64: both are callee-saved,
//!   both are saved/restored whole by `__rt_fiber_switch`, and both are excluded
//!   from the linear-scan allocator pools (`callee_int_pool` uses x21-x27 on
//!   AArch64 and rbx only on x86_64), so no register-allocated value ever
//!   collides with the ctx pointer.
//! - x18 is NOT used: Apple AArch64 reserves it for the OS.
//! - The layout embeds only the heap-path state for the spike: concat scratch
//!   (buf + off), heap bump offset, free-list head, and the small-bin heads.
//!   `_heap_buf` itself stays a global symbol during the spike because its base
//!   address is identical for every ctx and it is never reset per context.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Concat scratch capacity in bytes, mirroring `strings::CONCAT_BUF_CAPACITY`.
/// Duplicated here so the ctx layout is self-contained for the spike.
pub(crate) const CTX_CONCAT_BUF_CAPACITY: usize = 65536;

/// Small-bin head count, mirroring the 4 class heads emitted as `_heap_small_bins`.
pub(crate) const CTX_HEAP_SMALL_BIN_COUNT: usize = 4;

/// Byte offset of `_concat_off` inside `_rt_ctx`.
///
/// Scalar fields lead the layout and the 64 KiB concat buffer closes it: every
/// scalar offset stays small enough for the AArch64 unsigned-imm12 encoding
/// (`ldr xN, [x28, #imm]`), which caps at 4095.
pub(crate) const CTX_CONCAT_OFF_OFFSET: usize = 0;

/// Byte offset of `_heap_off` inside `_rt_ctx`.
pub(crate) const CTX_HEAP_OFF_OFFSET: usize = CTX_CONCAT_OFF_OFFSET + 8;

/// Byte offset of `_heap_free_list` inside `_rt_ctx`.
pub(crate) const CTX_HEAP_FREE_LIST_OFFSET: usize = CTX_HEAP_OFF_OFFSET + 8;

/// Byte offset of `_heap_small_bins` inside `_rt_ctx`.
pub(crate) const CTX_HEAP_SMALL_BINS_OFFSET: usize = CTX_HEAP_FREE_LIST_OFFSET + 8;

/// Byte offset of `_concat_buf` inside `_rt_ctx`.
///
/// The 64 KiB scratch buffer closes the layout: accesses derive its address
/// once via `emit_ctx_address` (an `add xN, x28, #imm` also within imm12) and
/// then index freely from there, so the large offset never reaches an
/// immediate-offset load.
pub(crate) const CTX_CONCAT_BUF_OFFSET: usize = CTX_HEAP_SMALL_BINS_OFFSET + CTX_HEAP_SMALL_BIN_COUNT * 8;

/// Total byte size of one `_rt_ctx` instance (16-byte aligned).
///
/// Covers the leading scalar fields, the four small-bin heads, and the closing
/// 64 KiB concat scratch buffer.
pub(crate) const CTX_SIZE: usize =
    (CTX_CONCAT_BUF_OFFSET + CTX_CONCAT_BUF_CAPACITY + 15) & !15;

/// Returns the reserved ctx-pointer register name for the target.
///
/// x28 (AArch64) / r14 (x86_64): callee-saved, saved whole by the fiber switch,
/// and excluded from both register-allocator pools, so the ctx pointer survives
/// every call and every fiber switch without extra spills.
pub fn ctx_reg(emitter: &Emitter) -> &'static str {
    match emitter.target.arch {
        Arch::AArch64 => "x28",
        Arch::X86_64 => "r14",
    }
}

/// Emits the `_rt_ctx` data block: one instance of the context struct.
///
/// Emitted as `.bss`-style `.comm` when `single` is true (the spike's default
/// executable mode owns exactly one context). A future pool emitter will replace
/// this with an array plus a free-list for per-thread contexts.
// The ctx data block currently ships through `runtime::data::fixed`'s string
// assembly path; this emitter-shaped variant stays for the pool iteration.
#[allow(dead_code)]
pub fn emit_rt_ctx_data(emitter: &mut Emitter, single: bool) {
    if single {
        emitter.raw(".data");
        emitter.raw(&format!(".globl _rt_ctx\n_rt_ctx:\n    .space {}", CTX_SIZE));
    }
}

/// Zeroes the heap allocator state (bump offset, free-list head, small-bin
/// heads), ctx-relative in ctx-register mode and via the legacy globals
/// otherwise.
///
/// Used by the `--web` per-request arena reset: the whole arena is reclaimed at
/// once after every refcounted per-request value has already been released.
pub fn emit_heap_arena_reset_state(emitter: &mut Emitter) {
    let fields = [
        CTX_HEAP_OFF_OFFSET,
        CTX_HEAP_FREE_LIST_OFFSET,
        CTX_HEAP_SMALL_BINS_OFFSET,
        CTX_HEAP_SMALL_BINS_OFFSET + 8,
        CTX_HEAP_SMALL_BINS_OFFSET + 16,
        CTX_HEAP_SMALL_BINS_OFFSET + 24,
    ];
    if emitter.ctx_register {
        for offset in fields {
            match emitter.target.arch {
                Arch::AArch64 => {
                    emitter.instruction(&format!("str xzr, [x28, #{}]", offset)); // zero one per-context heap allocator field
                }
                Arch::X86_64 => {
                    emitter.instruction(&format!("mov QWORD PTR [r14 + {}], 0", offset)); // zero one per-context heap allocator field
                }
            }
        }
        return;
    }
    abi::emit_store_zero_to_symbol(emitter, "_heap_off", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_free_list", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 0);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 8);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 16);
    abi::emit_store_zero_to_symbol(emitter, "_heap_small_bins", 24);
}

/// Publishes the `_rt_ctx` base into the reserved ctx register without a call.
///
/// Used by entry points that must (re-)establish the per-context state pointer
/// without the full zeroing pass of `__rt_ctx_init` — notably the fiber entry
/// trampoline, whose fresh stack arrives with a zeroed save area.
pub fn emit_ctx_publish(emitter: &mut Emitter) {
    abi::emit_symbol_address(emitter, ctx_reg(emitter), "_rt_ctx");
}

/// Zeroes every mutable ctx field (concat offset, heap bump, free list, bins)
/// through the already-published ctx register, on both targets.
///
/// Split out of `emit_rt_ctx_init` so library-entry points can reuse the
/// reset without emitting a call: `elephc_init` publishes the pointer inline
/// and must zero the same fields the helper does.
pub fn emit_ctx_zero_fields(emitter: &mut Emitter) {
    let ctx = ctx_reg(emitter);
    for offset in [
        CTX_CONCAT_OFF_OFFSET,
        CTX_HEAP_OFF_OFFSET,
        CTX_HEAP_FREE_LIST_OFFSET,
    ] {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!("str xzr, [{}, #{}]", ctx, offset)); // reset one mutable ctx field to zero
            }
            Arch::X86_64 => {
                emitter.instruction(&format!("mov QWORD PTR [{} + {}], 0", ctx, offset)); // reset one mutable ctx field to zero
            }
        }
    }
    for bin in 0..CTX_HEAP_SMALL_BIN_COUNT {
        let offset = CTX_HEAP_SMALL_BINS_OFFSET + bin * 8;
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!("str xzr, [{}, #{}]", ctx, offset)); // reset one small-bin head to empty
            }
            Arch::X86_64 => {
                emitter.instruction(&format!("mov QWORD PTR [{} + {}], 0", ctx, offset)); // reset one small-bin head to empty
            }
        }
    }
}

/// Emits `__rt_ctx_init`: publishes the `_rt_ctx` base into the ctx register and
/// zeroes the mutable ctx fields (concat offset, heap bump, free list, bins).
///
/// Contract:
/// - Must be called exactly once per execution context before any ctx-relative
///   access runs (main prologue today; a spawned thread's entry later). Calling
///   it again on a LIVE context resets the allocator (zeroed bump offset,
///   empty free list) — catastrophic mid-request, intended only at fresh
///   context boundaries (pool reuse, per-request reset).
/// - Clobber set, pinned: the ctx register (`x28`/`r14`) is REWRITTEN (that is
///   its purpose) and the AArch64 `adrp`/x86_64 `lea` sequences borrow the
///   standard symbol scratch (`x9` AArch64; none on x86_64 — RIP-relative).
///   No argument register (`x0`-`x7`, `rdi`-`r9`) is touched, so a prologue
///   may call this helper BEFORE argc/argv have been spilled without
///   corrupting them. Keep this property: new field stores must stay on the
///   ctx register, not on argument registers.
/// - The data section's `.space` zero-fill covers the initial zeroing, but the
///   explicit zero stores keep `__rt_ctx_init` correct for a REUSED context
///   (the M1 thread-pool case where a fresh request reuses a pooled ctx).
pub fn emit_rt_ctx_init(emitter: &mut Emitter) {
    let ctx = ctx_reg(emitter);
    emitter.blank();
    emitter.comment("--- runtime: ctx_init (publish per-context state pointer) ---");
    emitter.label_global("__rt_ctx_init");
    abi::emit_symbol_address(emitter, ctx, "_rt_ctx");
    // Zero the mutable fields so a REUSED context (thread pool, request reuse)
    // starts pristine even though the data section zero-fills only the first use.
    emit_ctx_zero_fields(emitter);
    emitter.instruction("ret");
}

/// Loads a ctx field into `reg` through the reserved ctx register.
///
/// `field_offset` must be one of the `CTX_*_OFFSET` constants from this module.
// Consumed by the ctx-gated heap/concat helper routing (next spike iteration).
#[allow(dead_code)]
pub fn emit_ctx_load(emitter: &mut Emitter, reg: &str, field_offset: usize) {
    let ctx = ctx_reg(emitter);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("ldr {}, [{}, #{}]", reg, ctx, field_offset));
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov {}, QWORD PTR [{} + {}]", reg, ctx, field_offset));
        }
    }
}

/// Stores `reg` into a ctx field through the reserved ctx register.
// Consumed by the ctx-gated heap/concat helper routing (next spike iteration).
#[allow(dead_code)]
pub fn emit_ctx_store(emitter: &mut Emitter, reg: &str, field_offset: usize) {
    let ctx = ctx_reg(emitter);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("str {}, [{}, #{}]", reg, ctx, field_offset));
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov QWORD PTR [{} + {}], {}", ctx, field_offset, reg));
        }
    }
}

/// Computes the address of a ctx field into `dest` through the ctx register.
// Consumed by the ctx-gated heap/concat helper routing (next spike iteration).
#[allow(dead_code)]
pub fn emit_ctx_address(emitter: &mut Emitter, dest: &str, field_offset: usize) {
    let ctx = ctx_reg(emitter);
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("add {}, {}, #{}", dest, ctx, field_offset));
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("lea {}, [{} + {}]", dest, ctx, field_offset));
        }
    }
}

/// Loads the free-list head value into `reg`, ctx-relative in ctx-register
/// mode and from the legacy global symbol otherwise.
///
/// Mirrors `emit_heap_off_load` for `_heap_free_list` so every helper that
/// consumes the free-list head shares one addressing-mode switch.
pub fn emit_free_list_head_load(emitter: &mut Emitter, reg: &str) {
    if emitter.ctx_register {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!(
                    "ldr {}, [x28, #{}]",
                    reg, CTX_HEAP_FREE_LIST_OFFSET
                )); // load the per-context free-list head
            }
            Arch::X86_64 => {
                emitter.instruction(&format!(
                    "mov {}, QWORD PTR [r14 + {}]",
                    reg, CTX_HEAP_FREE_LIST_OFFSET
                )); // load the per-context free-list head
            }
        }
        return;
    }
    abi::emit_symbol_address(emitter, reg, "_heap_free_list");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("ldr {}, [{}]", reg, reg)); // load the legacy global free-list head
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov {}, QWORD PTR [{}]", reg, reg)); // load the legacy global free-list head
        }
    }
}

/// Stores the heap bump offset value in `reg` back into per-context state,
/// ctx-relative in ctx-register mode and to the legacy global symbol otherwise.
///
/// Companion to `emit_heap_off_load` for the bump-shrink paths (tail trimming,
/// bump resets) that write the allocator cursor back. In legacy mode the value
/// is stored straight through the given scratch register: the caller must not
/// rely on it surviving.
pub fn emit_heap_off_store(emitter: &mut Emitter, reg: &str, value: &str) {
    if emitter.ctx_register {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!(
                    "str {}, [x28, #{}]",
                    value, CTX_HEAP_OFF_OFFSET
                )); // store the per-context heap bump offset
            }
            Arch::X86_64 => {
                emitter.instruction(&format!(
                    "mov QWORD PTR [r14 + {}], {}",
                    CTX_HEAP_OFF_OFFSET, value
                )); // store the per-context heap bump offset
            }
        }
        return;
    }
    abi::emit_symbol_address(emitter, reg, "_heap_off");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("str {}, [{}]", value, reg)); // store the legacy global heap offset
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov QWORD PTR [{}], {}", reg, value)); // store the legacy global heap offset
        }
    }
}

/// Materializes the free-list head SLOT address into `reg`, ctx-relative in
/// ctx-register mode and from the legacy global symbol otherwise.
///
/// Distinct from `emit_free_list_head_load` (which loads the head VALUE): the
/// ordered-insertion and merge paths keep a mutable pointer to the previous
/// next-slot, so they need the slot's address rather than its contents.
pub fn emit_free_list_address(emitter: &mut Emitter, reg: &str) {
    if emitter.ctx_register {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!(
                    "add {}, x28, #{}",
                    reg, CTX_HEAP_FREE_LIST_OFFSET
                )); // address of the per-context free-list head slot
            }
            Arch::X86_64 => {
                emitter.instruction(&format!(
                    "lea {}, [r14 + {}]",
                    reg, CTX_HEAP_FREE_LIST_OFFSET
                )); // address of the per-context free-list head slot
            }
        }
        return;
    }
    abi::emit_symbol_address(emitter, reg, "_heap_free_list");
}

/// Materializes the small-bin head array address into `reg`, ctx-relative in
/// ctx-register mode and from the legacy global symbol otherwise.
pub fn emit_small_bins_address(emitter: &mut Emitter, reg: &str) {
    if emitter.ctx_register {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!(
                    "add {}, x28, #{}",
                    reg, CTX_HEAP_SMALL_BINS_OFFSET
                )); // base of the per-context small-bin head array
            }
            Arch::X86_64 => {
                emitter.instruction(&format!(
                    "lea {}, [r14 + {}]",
                    reg, CTX_HEAP_SMALL_BINS_OFFSET
                )); // base of the per-context small-bin head array
            }
        }
        return;
    }
    abi::emit_symbol_address(emitter, reg, "_heap_small_bins");
}

/// Loads the current heap bump offset into `reg`, ctx-relative in ctx-register
/// mode and from the legacy global symbol otherwise.
///
/// This is the shared range-check front end: every refcount/deep-free helper
/// that validates `ptr < _heap_buf + _heap_off` goes through it, so the
/// ctx-mode runtime keeps one consistent source for the live heap end.
pub fn emit_heap_off_load(emitter: &mut Emitter, reg: &str) {
    if emitter.ctx_register {
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!(
                    "ldr {}, [x28, #{}]",
                    reg, CTX_HEAP_OFF_OFFSET
                )); // load the per-context heap bump offset
            }
            Arch::X86_64 => {
                emitter.instruction(&format!(
                    "mov {}, QWORD PTR [r14 + {}]",
                    reg, CTX_HEAP_OFF_OFFSET
                )); // load the per-context heap bump offset
            }
        }
        return;
    }
    abi::emit_symbol_address(emitter, reg, "_heap_off");
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction(&format!("ldr {}, [{}]", reg, reg)); // load the legacy global heap offset
        }
        Arch::X86_64 => {
            emitter.instruction(&format!("mov {}, QWORD PTR [{}]", reg, reg)); // load the legacy global heap offset
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_support::RuntimeFeatures;
    use crate::codegen_support::emit::Emitter;
    use crate::codegen_support::platform::{Arch, Platform, Target};

    /// `ctx_reg` resolves to the reserved callee-saved register on each target.
    #[test]
    fn ctx_register_is_reserved_callee_saved_per_target() {
        let arm = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        assert_eq!(ctx_reg(&arm), "x28");
        let x86 = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        assert_eq!(ctx_reg(&x86), "r14");
    }

    /// The ctx layout is self-consistent: scalars first, buffer last, every
    /// scalar offset inside the AArch64 imm12 window, size 16-byte aligned.
    #[test]
    fn ctx_layout_offsets_are_ordered_and_size_is_aligned() {
        // Scalars lead so their offsets stay within the unsigned-imm12 window.
        assert_eq!(CTX_CONCAT_OFF_OFFSET, 0);
        assert_eq!(CTX_HEAP_OFF_OFFSET, 8);
        assert_eq!(CTX_HEAP_FREE_LIST_OFFSET, 16);
        assert_eq!(CTX_HEAP_SMALL_BINS_OFFSET, 24);
        // The concat buffer closes the layout.
        assert_eq!(CTX_CONCAT_BUF_OFFSET, 24 + CTX_HEAP_SMALL_BIN_COUNT * 8);
        // Every scalar offset must be encodable as ldr [x28, #imm] (imm12 ≤ 4095).
        for offset in [
            CTX_CONCAT_OFF_OFFSET,
            CTX_HEAP_OFF_OFFSET,
            CTX_HEAP_FREE_LIST_OFFSET,
            CTX_HEAP_SMALL_BINS_OFFSET,
        ] {
            assert!(offset + 8 <= 4096, "scalar ctx offset {offset} escapes the imm12 window");
        }
        assert!(CTX_SIZE >= CTX_CONCAT_BUF_OFFSET + CTX_CONCAT_BUF_CAPACITY);
        assert_eq!(CTX_SIZE % 16, 0);
        // The concat scratch dominates the context, so a ctx instance is ~64 KiB.
        assert!(CTX_SIZE > CTX_CONCAT_BUF_CAPACITY);
    }

    /// `emit_rt_ctx_data` declares exactly one global `_rt_ctx` of `CTX_SIZE` bytes.
    #[test]
    fn rt_ctx_data_emits_single_global_instance() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_rt_ctx_data(&mut emitter, true);
        let asm = emitter.output();
        assert!(asm.contains(".globl _rt_ctx\n_rt_ctx:\n"));
        assert!(asm.contains(&format!(".space {}", CTX_SIZE)));
    }

    /// `__rt_ctx_init` publishes `_rt_ctx` into the ctx register and zeroes every
    /// mutable field on AArch64.
    #[test]
    fn ctx_init_publishes_pointer_and_zeroes_fields_aarch64() {
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_rt_ctx_init(&mut emitter);
        let asm = emitter.output();
        // Publishes the ctx base into x28 via the platform symbol helper (adrp+add).
        assert!(asm.contains("adrp x28, _rt_ctx"), "{asm}");
        assert!(asm.contains("add x28, x28, _rt_ctx"), "{asm}");
        // Zeroes concat off, heap off, free list, and all four small-bin heads.
        for offset in [
            CTX_CONCAT_OFF_OFFSET,
            CTX_HEAP_OFF_OFFSET,
            CTX_HEAP_FREE_LIST_OFFSET,
        ] {
            assert!(asm.contains(&format!("str xzr, [x28, #{}]", offset)), "{asm}");
        }
        for bin in 0..CTX_HEAP_SMALL_BIN_COUNT {
            let offset = CTX_HEAP_SMALL_BINS_OFFSET + bin * 8;
            assert!(asm.contains(&format!("str xzr, [x28, #{}]", offset)), "{asm}");
        }
    }

    /// `__rt_ctx_init` publishes `_rt_ctx` into r14 and zeroes every mutable field
    /// on x86_64.
    #[test]
    fn ctx_init_publishes_pointer_and_zeroes_fields_x86_64() {
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emit_rt_ctx_init(&mut emitter);
        let asm = emitter.output();
        assert!(asm.contains("lea r14, [rip + _rt_ctx]"), "{asm}");
        for offset in [
            CTX_CONCAT_OFF_OFFSET,
            CTX_HEAP_OFF_OFFSET,
            CTX_HEAP_FREE_LIST_OFFSET,
        ] {
            assert!(
                asm.contains(&format!("mov QWORD PTR [r14 + {}], 0", offset)),
                "{asm}"
            );
        }
        for bin in 0..CTX_HEAP_SMALL_BIN_COUNT {
            let offset = CTX_HEAP_SMALL_BINS_OFFSET + bin * 8;
            assert!(
                asm.contains(&format!("mov QWORD PTR [r14 + {}], 0", offset)),
                "{asm}"
            );
        }
    }

    /// Ctx-relative loads/stores/addresses go through the ctx register on both
    /// targets and never reference a global symbol.
    #[test]
    fn ctx_access_helpers_route_through_ctx_register() {
        for (target, scratch, load, store, address) in [
            (
                Target::new(Platform::MacOS, Arch::AArch64),
                "x9",
                "ldr x9, [x28, #72]",
                "str x9, [x28, #72]",
                "add x9, x28, #72",
            ),
            (
                Target::new(Platform::Linux, Arch::X86_64),
                "r9",
                "mov r9, QWORD PTR [r14 + 72]",
                "mov QWORD PTR [r14 + 72], r9",
                "lea r9, [r14 + 72]",
            ),
        ] {
            let mut emitter = Emitter::new(target);
            emit_ctx_load(&mut emitter, scratch, 72);
            emit_ctx_store(&mut emitter, scratch, 72);
            emit_ctx_address(&mut emitter, scratch, 72);
            let asm = emitter.output();
            assert!(asm.contains(load), "{asm}");
            assert!(asm.contains(store), "{asm}");
            assert!(asm.contains(address), "{asm}");
            assert!(!asm.contains("adrp"), "ctx access must not materialize symbols: {asm}");
        }
    }

    /// The shared heap-offset loader picks the addressing mode from the emitter:
    /// ctx-relative through x28/r14 in ctx mode, legacy symbol otherwise.
    #[test]
    fn heap_off_load_switches_addressing_mode() {
        // AArch64 ctx mode reads through x28 with no symbol materialization.
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emitter.ctx_register = true;
        emit_heap_off_load(&mut emitter, "x10");
        let asm = emitter.output();
        assert!(
            asm.contains(&format!("ldr x10, [x28, #{}]", CTX_HEAP_OFF_OFFSET)),
            "{asm}"
        );
        assert!(!asm.contains("adrp"), "ctx mode must not materialize symbols: {asm}");

        // AArch64 legacy mode keeps the adrp+add+ldr sequence against the global.
        let mut emitter = Emitter::new(Target::new(Platform::MacOS, Arch::AArch64));
        emit_heap_off_load(&mut emitter, "x10");
        let asm = emitter.output();
        assert!(asm.contains("adrp x10, _heap_off"), "{asm}");
        assert!(asm.contains("ldr x10, [x10]"), "{asm}");

        // x86_64 ctx mode reads through r14.
        let mut emitter = Emitter::new(Target::new(Platform::Linux, Arch::X86_64));
        emitter.ctx_register = true;
        emit_heap_off_load(&mut emitter, "r11");
        let asm = emitter.output();
        assert!(
            asm.contains(&format!("mov r11, QWORD PTR [r14 + {}]", CTX_HEAP_OFF_OFFSET)),
            "{asm}"
        );
    }
    /// The full generated runtime honors the ctx-register feature end to end:
    /// `__rt_ctx_init` is present, `__rt_heap_alloc` reads heap state through
    /// x28, and the `_rt_ctx` data block is declared.
    #[test]
    fn ctx_feature_generates_ctx_addressed_runtime_end_to_end() {
        use crate::codegen_support::driver_support::generate_runtime_with_features;
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        let asm = generate_runtime_with_features(
            8 * 1024 * 1024,
            target,
            RuntimeFeatures {
                ctx_register: true,
                ..RuntimeFeatures::none()
            },
        );
        // The publication helper is emitted and installs x28.
        assert!(asm.contains("__rt_ctx_init:"), "ctx runtime must emit __rt_ctx_init");
        assert!(asm.contains("adrp x28, _rt_ctx"), "{asm}");
        // The allocator reads per-context heap state through x28.
        assert!(
            asm.contains(&format!("ldr x10, [x28, #{}]", CTX_HEAP_OFF_OFFSET)),
            "ctx runtime allocator must bump through x28"
        );
        // The data block backs it.
        assert!(asm.contains(".globl _rt_ctx\n_rt_ctx:\n"), "{asm}");
        assert!(asm.contains(&format!(".space {}", CTX_SIZE)), "{asm}");
    }

    /// The legacy runtime is untouched by the feature: no `__rt_ctx_init`, no
    /// `_rt_ctx` block, and the allocator still materializes `_heap_off`.
    #[test]
    fn legacy_feature_keeps_symbol_addressed_runtime() {
        use crate::codegen_support::driver_support::generate_runtime_with_features;
        let target = Target::new(Platform::MacOS, Arch::AArch64);
        let asm = generate_runtime_with_features(8 * 1024 * 1024, target, RuntimeFeatures::none());
        assert!(!asm.contains("__rt_ctx_init"), "legacy runtime must not emit ctx helpers");
        assert!(!asm.contains("_rt_ctx"), "legacy runtime must not declare the ctx block");
        assert!(asm.contains("_heap_off"), "legacy runtime keeps symbol addressing");
    }

    /// Scratch audit for the reserved ctx register (spike review, B3): across
    /// the ENTIRE ctx-mode runtime text, the ctx register may only appear in
    /// sanctioned shapes — the publish/install sequences, ctx-relative
    /// loads/stores/addresses, and the fiber-switch save/restore pairs. Any
    /// hand-written helper that starts using x28/r14 as an ordinary scratch
    /// register would silently corrupt the per-context state pointer, and no
    /// other test would catch it; this one fails on the first stray shape.
    #[test]
    fn ctx_mode_runtime_never_scratches_the_ctx_register() {
        use crate::codegen_support::driver_support::generate_runtime_with_features;
        for (platform, arch, ctx_reg) in [
            (Platform::MacOS, Arch::AArch64, "x28"),
            (Platform::Linux, Arch::X86_64, "r14"),
        ] {
            let asm = generate_runtime_with_features(
                8 * 1024 * 1024,
                Target::new(platform, arch),
                RuntimeFeatures {
                    ctx_register: true,
                    ..RuntimeFeatures::none()
                },
            );
            let mut offenders: Vec<&str> = Vec::new();
            for line in asm.lines().filter_map(|line| line.trim().strip_prefix(';')) {
                let line = line.trim();
                if !line.contains(ctx_reg) {
                    continue;
                }
                // Sanctioned shapes per line:
                //  - save/restore pairs by the fiber switch (stp/ldp x27, x28 / push|pop r14)
                //  - publish/install (adrp x28, _rt_ctx / add x28, x28, _rt_ctx / lea r14, [rip + _rt_ctx])
                //  - ctx-relative access: ldr/str ... [x28, #imm] / mov ... [r14 + imm] / add xN, x28, #imm / lea rN, [r14 + imm]
                let sanctioned = match arch {
                    Arch::AArch64 => {
                        line.starts_with("stp x27, x28")
                            || line.starts_with("ldp x27, x28")
                            || line.starts_with("adrp x28, _rt_ctx")
                            || line.starts_with("add x28, x28, _rt_ctx")
                            || (line.starts_with("ldr ") && line.contains("[x28, #"))
                            || (line.starts_with("str ") && line.contains("[x28, #"))
                            || (line.starts_with("add x") && line.ends_with(&format!(", #{}", "")))
                            || (line.starts_with("add x") && line.contains(", x28, #"))
                    }
                    Arch::X86_64 => {
                        line.starts_with("push r14")
                            || line.starts_with("pop r14")
                            || line.starts_with("lea r14, [rip + _rt_ctx]")
                            || (line.starts_with("mov ") && line.contains(&format!("[{ctx_reg} +")))
                            || (line.starts_with("lea ") && line.contains(&format!("[{ctx_reg} +")))
                    }
                };
                if !sanctioned {
                    offenders.push(line);
                }
            }
            assert!(
                offenders.is_empty(),
                "{arch:?} ctx runtime uses {ctx_reg} outside sanctioned shapes ({}
                 offenders, e.g. {:?})",
                offenders.len(),
                offenders.first(),
            );
        }
    }
}