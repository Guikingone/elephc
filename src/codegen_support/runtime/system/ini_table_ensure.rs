//! Purpose:
//! Emits `__rt_ini_table_ensure`, the lazy one-shot seeder for the persistent
//! global PHP ini directive table backing `ini_get`/`ini_set`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::system`.
//! - `crate::codegen_support::runtime::data::fixed` reads `INI_SEED` to emit the seed literals.
//!
//! Key details:
//! - The table pointer lives in the low word of `.comm _rt_ini_table`; `_rt_ini_table_init`
//!   is a one-shot seed marker (0 = unseeded). Seeding builds a str-valued hash and inserts
//!   each seed directive with an OWNED (`__rt_str_persist`) value; keys are passed borrowed
//!   because `__rt_hash_set` persists keys itself but takes ownership of values.

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// The mutable ini directive table's CLI seed values (measured with `php -n`).
///
/// This is also the REGISTERED-directive set: `ini_set` only accepts directives
/// present here (PHP rejects unregistered directives, returning `false` and storing
/// nothing). `ini_get` returns these values until overwritten by `ini_set`; an
/// unseeded directive yields `false`, matching PHP both for genuinely unregistered
/// names and for extension directives (`session.*`/`opcache.*`/`apc.*`/`xdebug.*`)
/// that are not loaded — only CORE directives are seeded here. The tuple order is the
/// insertion order and drives the seed-literal symbol indices shared with
/// `crate::codegen_support::runtime::data::fixed`.
///
/// Two values intentionally diverge from a raw `php -n` `ini_get` reading to stay
/// consistent with the documented master-default choices: `error_reporting` holds the
/// numeric `E_ALL` (`30719`) table value (the documented not-live-synced limitation),
/// and `date.timezone` is seeded empty to mirror the master map that reports it absent.
pub(crate) const INI_SEED: &[(&str, &str)] = &[
    ("memory_limit", "128M"),
    ("max_execution_time", "0"),
    ("max_input_time", "-1"),
    ("display_errors", "1"),
    ("display_startup_errors", "1"),
    ("error_reporting", "30719"),
    ("log_errors", "0"),
    ("error_log", ""),
    ("default_charset", "UTF-8"),
    ("default_mimetype", "text/html"),
    ("precision", "14"),
    ("serialize_precision", "-1"),
    ("date.timezone", ""),
    ("output_buffering", "0"),
    ("zend.assertions", "1"),
    ("html_errors", "0"),
    ("implicit_flush", "1"),
    ("ignore_repeated_errors", "0"),
    ("variables_order", "EGPCS"),
    ("request_order", ""),
];

/// Returns the runtime data symbol name for the seed directive key at `index`.
///
/// Shared between the code emitter here and the `.ascii` data emitter in
/// `crate::codegen_support::runtime::data::fixed` so the address and length stay in sync.
pub(crate) fn ini_seed_key_symbol(index: usize) -> String {
    format!("_rt_ini_seed_key_{}", index)
}

/// Returns the runtime data symbol name for the seed directive value at `index`.
pub(crate) fn ini_seed_val_symbol(index: usize) -> String {
    format!("_rt_ini_seed_val_{}", index)
}

/// Emits `__rt_ini_table_ensure` for every supported target.
///
/// Lazily seeds the persistent ini directive table exactly once: on first call it
/// marks `_rt_ini_table_init`, allocates a str-valued hash, inserts every `INI_SEED`
/// directive with an owned persisted value, and publishes the table pointer into the
/// low word of `_rt_ini_table`. Subsequent calls observe the marker and return early.
pub fn emit_ini_table_ensure(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_ini_table_ensure_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: ini_table_ensure (lazy one-shot seed) ---");
    emitter.label_global("__rt_ini_table_ensure");

    // -- one-shot guard: return immediately once the table has been seeded --
    abi::emit_load_symbol_to_reg(emitter, "x9", "_rt_ini_table_init", 0);
    emitter.instruction("cbz x9, __rt_ini_table_ensure_seed");                  // seed the table on the first access only
    emitter.instruction("ret");                                                 // already seeded: nothing to do

    emitter.label("__rt_ini_table_ensure_seed");
    // Frame layout (48 bytes): [sp,#0]=hash ptr, [sp,#8]=owned value ptr,
    // [sp,#16]=owned value len, [sp,#32]=x29, [sp,#40]=x30.
    emitter.instruction("sub sp, sp, #48");                                     // allocate the ini seed frame
    emitter.instruction("stp x29, x30, [sp, #32]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #32");                                    // establish the new frame pointer

    // -- publish the seed marker first so a re-entrant call cannot double-seed --
    emitter.instruction("mov x9, #1");                                          // mark the ini table as seeded
    abi::emit_store_reg_to_symbol(emitter, "x9", "_rt_ini_table_init", 0);

    // -- allocate the str-valued directive hash (capacity 16) --
    emitter.instruction("mov x0, #16");                                         // initial capacity for the directive table
    emitter.instruction("mov x1, #1");                                          // value type = Str
    abi::emit_call_label(emitter, "__rt_hash_new");
    emitter.instruction("str x0, [sp, #0]");                                    // save the freshly allocated hash pointer

    for (index, (key, value)) in INI_SEED.iter().enumerate() {
        emitter.comment(&format!("-- seed directive: {} --", key));
        // Build an OWNED string value; the hash takes ownership of the value payload.
        abi::emit_symbol_address(emitter, "x1", &ini_seed_val_symbol(index));
        emitter.instruction(&format!("mov x2, #{}", value.len()));              // seed value literal length
        abi::emit_call_label(emitter, "__rt_str_persist");
        emitter.instruction("str x1, [sp, #8]");                                // save the owned value pointer
        emitter.instruction("str x2, [sp, #16]");                               // save the owned value length
        // Insert the directive; the key is borrowed because hash_set persists keys itself.
        abi::emit_symbol_address(emitter, "x1", &ini_seed_key_symbol(index));
        emitter.instruction(&format!("mov x2, #{}", key.len()));                // seed key literal length
        emitter.instruction("ldr x3, [sp, #8]");                                // value_lo = owned value pointer
        emitter.instruction("ldr x4, [sp, #16]");                               // value_hi = owned value length
        emitter.instruction("mov x5, #1");                                      // value tag = Str
        emitter.instruction("ldr x0, [sp, #0]");                                // hash pointer for the insert
        abi::emit_call_label(emitter, "__rt_hash_set");
        emitter.instruction("str x0, [sp, #0]");                                // persist any post-grow hash pointer
    }

    // -- publish the completed table pointer into the low word of _rt_ini_table --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the final hash pointer
    abi::emit_store_reg_to_symbol(emitter, "x0", "_rt_ini_table", 0);
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #48");                                     // deallocate the ini seed frame
    emitter.instruction("ret");                                                 // return to the caller
}

/// Emits `__rt_ini_table_ensure` for x86_64 Linux targets.
///
/// System V variant of `emit_ini_table_ensure`: seeds the same persistent ini table
/// using `rax`/`rdx` string conventions and the `rdi`-input `__rt_str_persist` helper.
fn emit_ini_table_ensure_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: ini_table_ensure (lazy one-shot seed) ---");
    emitter.label_global("__rt_ini_table_ensure");

    // -- one-shot guard: return immediately once the table has been seeded --
    abi::emit_load_symbol_to_reg(emitter, "r11", "_rt_ini_table_init", 0);
    emitter.instruction("test r11, r11");                                       // has the ini table already been seeded?
    emitter.instruction("jz __rt_ini_table_ensure_seed");                       // seed the table on the first access only
    emitter.instruction("ret");                                                 // already seeded: nothing to do

    emitter.label("__rt_ini_table_ensure_seed");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 48");                                         // reserve slots for hash ptr and owned value ptr/len

    // -- publish the seed marker first so a re-entrant call cannot double-seed --
    emitter.instruction("mov r11, 1");                                          // mark the ini table as seeded
    abi::emit_store_reg_to_symbol(emitter, "r11", "_rt_ini_table_init", 0);

    // -- allocate the str-valued directive hash (capacity 16) --
    emitter.instruction("mov rdi, 16");                                         // initial capacity for the directive table
    emitter.instruction("mov rsi, 1");                                          // value type = Str
    abi::emit_call_label(emitter, "__rt_hash_new");
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the freshly allocated hash pointer

    for (index, (key, value)) in INI_SEED.iter().enumerate() {
        emitter.comment(&format!("-- seed directive: {} --", key));
        // Build an OWNED string value; the hash takes ownership of the value payload.
        abi::emit_symbol_address(emitter, "rdi", &ini_seed_val_symbol(index));
        emitter.instruction(&format!("mov rdx, {}", value.len()));              // seed value literal length
        abi::emit_call_label(emitter, "__rt_str_persist");
        emitter.instruction("mov QWORD PTR [rbp - 16], rax");                   // save the owned value pointer
        emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                   // save the owned value length
        // Insert the directive; the key is borrowed because hash_set persists keys itself.
        emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                    // hash pointer for the insert
        abi::emit_symbol_address(emitter, "rsi", &ini_seed_key_symbol(index));
        emitter.instruction(&format!("mov rdx, {}", key.len()));                // seed key literal length
        emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                   // value_lo = owned value pointer
        emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                    // value_hi = owned value length
        emitter.instruction("mov r9, 1");                                       // value tag = Str
        abi::emit_call_label(emitter, "__rt_hash_set");
        emitter.instruction("mov QWORD PTR [rbp - 8], rax");                    // persist any post-grow hash pointer
    }

    // -- publish the completed table pointer into the low word of _rt_ini_table --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the final hash pointer
    abi::emit_store_reg_to_symbol(emitter, "rax", "_rt_ini_table", 0);
    emitter.instruction("add rsp, 48");                                         // deallocate the ini seed frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return to the caller
}
