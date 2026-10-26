//! Purpose:
//! Keeps compiler-private logical heap roots observable while next retires their owners.
//!
//! Called from:
//! - Generated SPL methods through typed runtime operations and allocator invalidation.
//!
//! Key details:
//! - Records are keyed by a live receiver and never stored in PHP-visible properties.
//! - Object, container and resource roots are borrowed; string copies own only inert bytes.
//! - The list supports removal out of order when different fibers suspend their mutations.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

mod capture;
mod read;
mod retire;
mod invalidate;
mod clone;
mod root_release;
mod properties;

const HEAD: &str = "_spl_heap_scope_head";
const SIZE: i64 = 64;
const QUEUE: usize = 56;
const CLONE_HOOK: &str = "_spl_heap_scope_clone_fn";
const OBJECT: usize = 8;
const DATA: usize = 16;
const PRIORITY: usize = 24;
const ACTIVE: usize = 32;
const DATA_VALID: usize = 40;
const PRIORITY_VALID: usize = 48;

/// Emits every scope helper using the same record layout on all supported targets.
pub fn emit_heap_scopes(e: &mut Emitter) {
    emit_find(e);
    capture::emit(e);
    read::emit(e);
    retire::emit(e);
    invalidate::emit(e);
    clone::emit(e);
    root_release::emit(e);
    properties::emit(e);
}

/// Observes a validated free while preserving the allocator working state.
pub(super) fn emit_free_observer(e: &mut Emitter) { invalidate::emit_free_observer(e); }

/// Emits one target instruction for the logical heap-scope state machine.
fn ins(e: &mut Emitter, arm: &str, x86: &str) {
    e.instruction(if arm64(e) { arm } else { x86 });                            // perform the selected target's heap-scope bookkeeping step
}

/// Reports the architecture without treating Apple and Linux ARM64 differently.
fn arm64(e: &Emitter) -> bool { e.target.arch == Arch::AArch64 }

/// Returns the custom runtime integer result register.
fn result(e: &Emitter) -> &'static str { abi::int_result_reg(e) }

/// Loads a saved operand into a standard C argument register.
fn arg(e: &mut Emitter, index: usize, offset: usize) {
    abi::emit_load_temporary_stack_slot(e, abi::int_arg_reg_name(e.target, index), offset);
}

/// Branches on zero without borrowing or overwriting another live register.
fn zero(e: &mut Emitter, register: &str, label: &str) {
    if arm64(e) {
        e.instruction(&format!("cbz {register}, {label}"));                     // skip the absent scope or inactive borrowed root
    } else {
        e.instruction(&format!("test {register}, {register}"));                 // test the pointer or flag without changing its value
        e.instruction(&format!("jz {label}"));                                  // skip the absent scope or inactive borrowed root
    }
}

/// Compares two pointers or logical words, retaining both input registers.
fn equal(e: &mut Emitter, left: &str, right: &str, label: &str) {
    ins(e, &format!("cmp {left}, {right}"), &format!("cmp {left}, {right}"));
    ins(e, &format!("b.eq {label}"), &format!("je {label}"));
}

/// Branches when a saved runtime tag equals one concrete PHP value category.
fn tag_is(e: &mut Emitter, register: &str, tag: usize, label: &str) {
    ins(e, &format!("cmp {register}, #{tag}"), &format!("cmp {register}, {tag}"));
    ins(e, &format!("b.eq {label}"), &format!("je {label}"));
}

/// Finds a receiver's active record without acquiring any object or value owner.
fn emit_find(e: &mut Emitter) {
    e.label_global("__rt_heap_scope_find");
    let value = result(e);
    let receiver = if arm64(e) { "x15" } else { "r8" };
    let candidate = if arm64(e) { "x11" } else { "r11" };
    abi::emit_reg_move(e, receiver, abi::int_arg_reg_name(e.target, 0));
    abi::emit_load_symbol_to_reg(e, value, HEAD, 0);
    e.label("__rt_heap_scope_find_loop");
    zero(e, value, "__rt_heap_scope_find_done");
    abi::emit_load_from_address(e, candidate, value, OBJECT);
    equal(e, candidate, receiver, "__rt_heap_scope_find_done");
    abi::emit_load_from_address(e, value, value, 0);
    abi::emit_jump(e, "__rt_heap_scope_find_loop");
    e.label("__rt_heap_scope_find_done");
    abi::emit_return(e);
}
