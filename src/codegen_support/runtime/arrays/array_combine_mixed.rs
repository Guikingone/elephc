//! Purpose:
//! Emits `__rt_array_combine_mixed`, the gradual-typing `array_combine()` runtime helper that
//! pairs two heterogeneous (Mixed / Array(Mixed)) operands positionally into an associative
//! array, matching PHP semantics: count mismatch throws a catchable `\ValueError`, keys are
//! coerced exactly as PHP does (integers stay integer keys; every other type is `(string)`-cast
//! then normalized so a numeric string becomes an integer key), and values are preserved as
//! boxed Mixed entries.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//! - The generated code from `crate::codegen::lower_inst::builtins::arrays::lower_array_combine`
//!   (gradual path), which boxes each operand into a Mixed cell before the call.
//!
//! Key details:
//! - Both operands are normalized to freshly OWNED hashes via `__rt_mixed_to_owned_hash`, then
//!   walked in parallel by insertion order. Each entry's VALUE (not its key) is the pairing
//!   datum: the keys-operand values become result keys, the values-operand values become result
//!   values. This makes indexed and associative operands interchangeable at runtime.
//! - Key coercion delegates to `__rt_mixed_cast_string` + `__rt_hash_normalize_key` for every
//!   non-integer key, which reproduces PHP's array_combine coercion (float `1.9` → string `"1.9"`,
//!   float `5.0` → int `5`, bool → `"1"`/`""`, null → `""`, numeric strings → int).
//! - Ownership balances exactly: values are retained into fresh boxed cells (tag-7 entries are
//!   increffed, typed entries reboxed via `__rt_mixed_from_value`); the two temp hashes are
//!   deep-freed with `__rt_decref_hash` after the walk, and per-iteration key temporaries
//!   (the cast string and its boxed source) are released. Verified `--heap-debug` clean.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

use super::value_error;

/// Byte length of the shared `_array_combine_count_msg` `ValueError` message.
const ARRAY_COMBINE_COUNT_MSG_LEN: usize =
    "array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements".len();

/// Emits the `__rt_array_combine_mixed` runtime helper for the active target.
///
/// Input:  `x0`/`rdi` = boxed Mixed keys cell, `x1`/`rsi` = boxed Mixed values cell.
/// Output: `x0`/`rax` = a freshly owned associative-array hash (value_type 7 = boxed Mixed).
pub fn emit_array_combine_mixed(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_combine_mixed_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_combine_mixed ---");
    emitter.label_global("__rt_array_combine_mixed");
    // Frame: [sp,#0]=keys hash, [sp,#8]=values hash, [sp,#16]=result hash,
    //        [sp,#24]=keys cursor, [sp,#32]=values cursor, [sp,#40]=key source tag,
    //        [sp,#48]=key source lo, [sp,#56]=key source hi, [sp,#64]=owned value cell,
    //        [sp,#72]=coerced key lo, [sp,#80]=coerced key hi, [sp,#88]=cast-string temp,
    //        [sp,#96]=boxed-key temp, [sp,#104]=saved values cell, saved fp/lr at [sp,#112].
    emitter.instruction("sub sp, sp, #128");                                    // reserve the combine frame plus saved fp/lr
    emitter.instruction("stp x29, x30, [sp, #112]");                            // save frame pointer and return address
    emitter.instruction("add x29, sp, #112");                                   // establish the local frame
    emitter.instruction("str x1, [sp, #104]");                                  // preserve the values cell across the first normalization
    emitter.instruction("bl __rt_mixed_to_owned_hash");                         // x0 = owned keys hash (refcount 1)
    emitter.instruction("str x0, [sp, #0]");                                    // save the owned keys hash
    emitter.instruction("ldr x0, [sp, #104]");                                  // reload the values cell
    emitter.instruction("bl __rt_mixed_to_owned_hash");                         // x0 = owned values hash (refcount 1)
    emitter.instruction("str x0, [sp, #8]");                                    // save the owned values hash
    // -- count check: mismatched element counts throw a catchable ValueError --
    emitter.instruction("ldr x9, [sp, #0]");                                    // keys hash pointer
    emitter.instruction("ldr x10, [x9]");                                       // x10 = keys count
    emitter.instruction("ldr x11, [sp, #8]");                                   // values hash pointer
    emitter.instruction("ldr x12, [x11]");                                      // x12 = values count
    emitter.instruction("cmp x10, x12");                                        // do both operands have the same element count?
    emitter.instruction("b.ne __rt_array_combine_mixed_mismatch");              // throw ValueError on a count mismatch
    // -- allocate the result hash (value_type 7 = boxed Mixed), capacity >= 16 --
    emitter.instruction("lsl x0, x10, #1");                                     // capacity = count * 2 for insertion headroom
    emitter.instruction("mov x9, #16");                                         // clamp to the minimum hash capacity
    emitter.instruction("cmp x0, x9");                                          // is the doubled count below the minimum?
    emitter.instruction("csel x0, x9, x0, lt");                                 // use 16 when the doubled count is too small
    emitter.instruction("mov x1, #7");                                          // value_type 7 = boxed Mixed entries
    emitter.instruction("bl __rt_hash_new");                                    // allocate the result hash
    emitter.instruction("str x0, [sp, #16]");                                   // save the result hash pointer
    emitter.instruction("str xzr, [sp, #24]");                                  // keys cursor = 0
    emitter.instruction("str xzr, [sp, #32]");                                  // values cursor = 0
    // -- walk both hashes in parallel by insertion order --
    emitter.label("__rt_array_combine_mixed_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // keys hash pointer
    emitter.instruction("ldr x1, [sp, #24]");                                   // keys cursor
    emitter.instruction("bl __rt_hash_iter_next");                             // x0=next cursor, x3/x4/x5 = this entry's value (the key source)
    emitter.instruction("cmn x0, #1");                                          // end of the keys walk?
    emitter.instruction("b.eq __rt_array_combine_mixed_done");                  // finish once every pair is combined
    emitter.instruction("str x0, [sp, #24]");                                   // save the next keys cursor
    emitter.instruction("str x5, [sp, #40]");                                   // save the key-source value tag
    emitter.instruction("str x3, [sp, #48]");                                   // save the key-source value low word
    emitter.instruction("str x4, [sp, #56]");                                   // save the key-source value high word
    emitter.instruction("ldr x0, [sp, #8]");                                    // values hash pointer
    emitter.instruction("ldr x1, [sp, #32]");                                   // values cursor
    emitter.instruction("bl __rt_hash_iter_next");                             // x0=next cursor, x3/x4/x5 = this entry's value (the result value)
    emitter.instruction("str x0, [sp, #32]");                                   // save the next values cursor
    // -- own the result value: retain a tag-7 cell, else rebox the typed value --
    emitter.instruction("cmp x5, #7");                                          // is the values-operand entry already a boxed Mixed cell?
    emitter.instruction("b.eq __rt_array_combine_mixed_val_boxed");             // retain the stored cell instead of double-boxing it
    emitter.instruction("mov x0, x5");                                          // mixed_from_value tag argument
    emitter.instruction("mov x1, x3");                                          // mixed_from_value low word argument
    emitter.instruction("mov x2, x4");                                          // mixed_from_value high word argument
    emitter.instruction("bl __rt_mixed_from_value");                           // rebox the typed value into an owned Mixed cell
    emitter.instruction("str x0, [sp, #64]");                                   // save the owned value cell
    emitter.instruction("b __rt_array_combine_mixed_have_val");                 // continue to key coercion
    emitter.label("__rt_array_combine_mixed_val_boxed");
    emitter.instruction("mov x0, x3");                                          // the stored cell is the value payload
    emitter.instruction("bl __rt_incref");                                      // retain it so the result owns its own reference
    emitter.instruction("str x0, [sp, #64]");                                   // save the owned value cell
    emitter.label("__rt_array_combine_mixed_have_val");
    emitter.instruction("str xzr, [sp, #88]");                                  // clear the cast-string cleanup marker
    emitter.instruction("str xzr, [sp, #96]");                                  // clear the boxed-key cleanup marker
    // -- coerce the key source into a normalized array key --
    emitter.instruction("ldr x5, [sp, #40]");                                   // reload the key-source value tag
    emitter.instruction("ldr x3, [sp, #48]");                                   // reload the key-source value low word
    emitter.instruction("ldr x4, [sp, #56]");                                   // reload the key-source value high word
    emitter.instruction("cmp x5, #7");                                          // is the key source a boxed Mixed cell?
    emitter.instruction("b.ne __rt_array_combine_mixed_key_typed");             // typed key sources are used directly
    emitter.instruction("mov x0, x3");                                          // unbox the stored Mixed key cell
    emitter.instruction("bl __rt_mixed_unbox");                                // x0=tag, x1=lo, x2=hi
    emitter.instruction("mov x5, x0");                                          // real key tag
    emitter.instruction("mov x3, x1");                                          // real key low word
    emitter.instruction("mov x4, x2");                                          // real key high word
    emitter.label("__rt_array_combine_mixed_key_typed");
    emitter.instruction("cmp x5, #0");                                          // integer keys stay integer keys
    emitter.instruction("b.ne __rt_array_combine_mixed_key_notint");            // non-integer keys need string coercion
    emitter.instruction("str x3, [sp, #72]");                                   // key_lo = the integer key value
    emitter.instruction("mov x9, #-1");                                         // key_hi = -1 marks an integer key
    emitter.instruction("str x9, [sp, #80]");                                   // save the integer key high word
    emitter.instruction("b __rt_array_combine_mixed_key_ready");                // proceed to the insert
    emitter.label("__rt_array_combine_mixed_key_notint");
    emitter.instruction("cmp x5, #1");                                          // string keys only need normalization
    emitter.instruction("b.ne __rt_array_combine_mixed_key_other");            // float/bool/null/array keys cast to string first
    emitter.instruction("mov x1, x3");                                          // normalize_key string pointer
    emitter.instruction("mov x2, x4");                                          // normalize_key string length
    emitter.instruction("bl __rt_hash_normalize_key");                        // x1=key_lo, x2=key_hi (numeric strings become integer keys)
    emitter.instruction("str x1, [sp, #72]");                                   // save the normalized key low word
    emitter.instruction("str x2, [sp, #80]");                                   // save the normalized key high word
    emitter.instruction("b __rt_array_combine_mixed_key_ready");                // proceed to the insert
    emitter.label("__rt_array_combine_mixed_key_other");
    emitter.instruction("mov x0, x5");                                          // mixed_from_value tag argument
    emitter.instruction("mov x1, x3");                                          // mixed_from_value low word argument
    emitter.instruction("mov x2, x4");                                          // mixed_from_value high word argument
    emitter.instruction("bl __rt_mixed_from_value");                           // box the key source so it can be string-cast
    emitter.instruction("str x0, [sp, #96]");                                   // save the boxed key for cleanup
    emitter.instruction("bl __rt_mixed_cast_string");                          // x1=string pointer, x2=string length (PHP (string) cast)
    emitter.instruction("str x1, [sp, #88]");                                   // save the cast string for cleanup
    emitter.instruction("bl __rt_hash_normalize_key");                        // x1=key_lo, x2=key_hi (numeric strings become integer keys)
    emitter.instruction("str x1, [sp, #72]");                                   // save the normalized key low word
    emitter.instruction("str x2, [sp, #80]");                                   // save the normalized key high word
    emitter.label("__rt_array_combine_mixed_key_ready");
    // -- insert the coerced key + owned value into the result hash --
    emitter.instruction("ldr x0, [sp, #16]");                                   // result hash pointer
    emitter.instruction("ldr x1, [sp, #72]");                                   // coerced key low word
    emitter.instruction("ldr x2, [sp, #80]");                                   // coerced key high word
    emitter.instruction("ldr x3, [sp, #64]");                                   // owned value cell
    emitter.instruction("mov x4, #0");                                          // boxed Mixed payloads use no high word
    emitter.instruction("mov x5, #7");                                          // value_type 7 = boxed Mixed
    emitter.instruction("bl __rt_hash_set");                                    // insert or overwrite the pair (persists a string-key copy)
    emitter.instruction("str x0, [sp, #16]");                                   // save the possibly-grown result hash
    // -- release the per-iteration key temporaries --
    emitter.instruction("ldr x0, [sp, #88]");                                   // cast-string temp (0 unless the key was string-cast)
    emitter.instruction("cbz x0, __rt_array_combine_mixed_no_str");             // skip when no cast string was allocated
    emitter.instruction("bl __rt_decref_any");                                  // free the cast string (hash_set kept its own copy)
    emitter.label("__rt_array_combine_mixed_no_str");
    emitter.instruction("ldr x0, [sp, #96]");                                   // boxed-key temp (0 unless the key was reboxed)
    emitter.instruction("cbz x0, __rt_array_combine_mixed_no_kbox");            // skip when no boxed key was allocated
    emitter.instruction("bl __rt_decref_mixed");                                // free the boxed key source
    emitter.label("__rt_array_combine_mixed_no_kbox");
    emitter.instruction("b __rt_array_combine_mixed_loop");                     // combine the next pair
    // -- done: deep-free the temp hashes and return the result --
    emitter.label("__rt_array_combine_mixed_done");
    emitter.instruction("ldr x0, [sp, #0]");                                    // keys hash
    emitter.instruction("bl __rt_decref_hash");                                 // deep-free the owned keys hash
    emitter.instruction("ldr x0, [sp, #8]");                                    // values hash
    emitter.instruction("bl __rt_decref_hash");                                 // deep-free the owned values hash
    emitter.instruction("ldr x0, [sp, #16]");                                   // x0 = the combined result hash
    emitter.instruction("ldp x29, x30, [sp, #112]");                           // restore frame pointer and return address
    emitter.instruction("add sp, sp, #128");                                   // release the combine frame
    emitter.instruction("ret");                                                // return the combined hash
    // -- count mismatch: free the temp hashes then throw a catchable ValueError --
    emitter.label("__rt_array_combine_mixed_mismatch");
    emitter.instruction("ldr x0, [sp, #0]");                                    // keys hash
    emitter.instruction("bl __rt_decref_hash");                                 // deep-free the owned keys hash before throwing
    emitter.instruction("ldr x0, [sp, #8]");                                    // values hash
    emitter.instruction("bl __rt_decref_hash");                                 // deep-free the owned values hash before throwing
    value_error::emit_throw_value_error_aarch64(
        emitter,
        "_array_combine_count_msg",
        ARRAY_COMBINE_COUNT_MSG_LEN,
    );
}

/// Emits the x86_64 Linux variant of `__rt_array_combine_mixed`.
///
/// Input:  `rdi` = boxed Mixed keys cell, `rsi` = boxed Mixed values cell.
/// Output: `rax` = a freshly owned associative-array hash (value_type 7 = boxed Mixed).
fn emit_array_combine_mixed_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_combine_mixed ---");
    emitter.label_global("__rt_array_combine_mixed");
    // Frame: [rbp-8]=keys hash, [rbp-16]=values hash, [rbp-24]=result hash,
    //        [rbp-32]=keys cursor, [rbp-40]=values cursor, [rbp-48]=key source tag,
    //        [rbp-56]=key source lo, [rbp-64]=key source hi, [rbp-72]=owned value cell,
    //        [rbp-80]=coerced key lo, [rbp-88]=coerced key hi, [rbp-96]=cast-string temp,
    //        [rbp-104]=boxed-key temp, [rbp-112]=saved values cell.
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 112");                                        // reserve the combine spill slots
    emitter.instruction("mov QWORD PTR [rbp - 112], rsi");                      // preserve the values cell across the first normalization
    emitter.instruction("call __rt_mixed_to_owned_hash");                       // rax = owned keys hash (rdi already set)
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the owned keys hash
    emitter.instruction("mov rdi, QWORD PTR [rbp - 112]");                      // reload the values cell
    emitter.instruction("call __rt_mixed_to_owned_hash");                       // rax = owned values hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the owned values hash
    // -- count check --
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // keys hash pointer
    emitter.instruction("mov r10, QWORD PTR [r10]");                            // r10 = keys count
    emitter.instruction("mov r11, QWORD PTR [rbp - 16]");                       // values hash pointer
    emitter.instruction("mov r11, QWORD PTR [r11]");                            // r11 = values count
    emitter.instruction("cmp r10, r11");                                        // do both operands have the same element count?
    emitter.instruction("jne __rt_array_combine_mixed_mismatch");               // throw ValueError on a count mismatch
    // -- allocate the result hash --
    emitter.instruction("lea rdi, [r10 + r10]");                                // capacity = count * 2 for insertion headroom
    emitter.instruction("cmp rdi, 16");                                         // is the doubled count below the minimum?
    emitter.instruction("jge __rt_array_combine_mixed_cap_ready");              // keep the doubled count when large enough
    emitter.instruction("mov rdi, 16");                                         // clamp to the minimum hash capacity
    emitter.label("__rt_array_combine_mixed_cap_ready");
    emitter.instruction("mov rsi, 7");                                          // value_type 7 = boxed Mixed entries
    emitter.instruction("call __rt_hash_new");                                  // allocate the result hash
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the result hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // keys cursor = 0
    emitter.instruction("mov QWORD PTR [rbp - 40], 0");                         // values cursor = 0
    // -- walk both hashes in parallel --
    emitter.label("__rt_array_combine_mixed_loop");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // keys hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // keys cursor
    emitter.instruction("call __rt_hash_iter_next");                           // rax=next cursor, rcx/r8/r9 = this entry's value (key source)
    emitter.instruction("cmp rax, -1");                                         // end of the keys walk?
    emitter.instruction("je __rt_array_combine_mixed_done");                    // finish once every pair is combined
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // save the next keys cursor
    emitter.instruction("mov QWORD PTR [rbp - 48], r9");                        // save the key-source value tag
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                       // save the key-source value low word
    emitter.instruction("mov QWORD PTR [rbp - 64], r8");                        // save the key-source value high word
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // values hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // values cursor
    emitter.instruction("call __rt_hash_iter_next");                           // rax=next cursor, rcx/r8/r9 = this entry's value (result value)
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the next values cursor
    // -- own the result value --
    emitter.instruction("cmp r9, 7");                                           // is the values-operand entry already a boxed Mixed cell?
    emitter.instruction("je __rt_array_combine_mixed_val_boxed");               // retain the stored cell instead of double-boxing it
    emitter.instruction("mov rax, r9");                                         // mixed_from_value tag argument
    emitter.instruction("mov rdi, rcx");                                        // mixed_from_value low word argument
    emitter.instruction("mov rsi, r8");                                         // mixed_from_value high word argument
    emitter.instruction("call __rt_mixed_from_value");                          // rebox the typed value into an owned Mixed cell
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // save the owned value cell
    emitter.instruction("jmp __rt_array_combine_mixed_have_val");               // continue to key coercion
    emitter.label("__rt_array_combine_mixed_val_boxed");
    emitter.instruction("mov rax, rcx");                                        // the stored cell is the value payload
    emitter.instruction("call __rt_incref");                                    // retain it so the result owns its own reference
    emitter.instruction("mov QWORD PTR [rbp - 72], rax");                       // save the owned value cell
    emitter.label("__rt_array_combine_mixed_have_val");
    emitter.instruction("mov QWORD PTR [rbp - 96], 0");                         // clear the cast-string cleanup marker
    emitter.instruction("mov QWORD PTR [rbp - 104], 0");                        // clear the boxed-key cleanup marker
    // -- coerce the key source into a normalized array key --
    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // reload the key-source value tag
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // reload the key-source value low word
    emitter.instruction("mov r8, QWORD PTR [rbp - 64]");                        // reload the key-source value high word
    emitter.instruction("cmp r9, 7");                                           // is the key source a boxed Mixed cell?
    emitter.instruction("jne __rt_array_combine_mixed_key_typed");              // typed key sources are used directly
    emitter.instruction("mov rdi, rcx");                                        // unbox the stored Mixed key cell
    emitter.instruction("call __rt_mixed_unbox");                              // rax=tag, rdi=lo, rdx=hi
    emitter.instruction("mov r9, rax");                                         // real key tag
    emitter.instruction("mov rcx, rdi");                                        // real key low word
    emitter.instruction("mov r8, rdx");                                         // real key high word
    emitter.label("__rt_array_combine_mixed_key_typed");
    emitter.instruction("cmp r9, 0");                                           // integer keys stay integer keys
    emitter.instruction("jne __rt_array_combine_mixed_key_notint");             // non-integer keys need string coercion
    emitter.instruction("mov QWORD PTR [rbp - 80], rcx");                       // key_lo = the integer key value
    emitter.instruction("mov QWORD PTR [rbp - 88], -1");                        // key_hi = -1 marks an integer key
    emitter.instruction("jmp __rt_array_combine_mixed_key_ready");              // proceed to the insert
    emitter.label("__rt_array_combine_mixed_key_notint");
    emitter.instruction("cmp r9, 1");                                           // string keys only need normalization
    emitter.instruction("jne __rt_array_combine_mixed_key_other");             // float/bool/null/array keys cast to string first
    emitter.instruction("mov rax, rcx");                                        // normalize_key string pointer
    emitter.instruction("mov rdx, r8");                                         // normalize_key string length
    emitter.instruction("call __rt_hash_normalize_key");                       // rax=key_lo, rdx=key_hi (numeric strings become integer keys)
    emitter.instruction("mov QWORD PTR [rbp - 80], rax");                       // save the normalized key low word
    emitter.instruction("mov QWORD PTR [rbp - 88], rdx");                       // save the normalized key high word
    emitter.instruction("jmp __rt_array_combine_mixed_key_ready");              // proceed to the insert
    emitter.label("__rt_array_combine_mixed_key_other");
    emitter.instruction("mov rax, r9");                                         // mixed_from_value tag argument
    emitter.instruction("mov rdi, rcx");                                        // mixed_from_value low word argument
    emitter.instruction("mov rsi, r8");                                         // mixed_from_value high word argument
    emitter.instruction("call __rt_mixed_from_value");                          // box the key source so it can be string-cast
    emitter.instruction("mov QWORD PTR [rbp - 104], rax");                      // save the boxed key for cleanup
    emitter.instruction("mov rdi, rax");                                        // mixed_cast_string input cell
    emitter.instruction("call __rt_mixed_cast_string");                        // rax=string pointer, rdx=string length (PHP (string) cast)
    emitter.instruction("mov QWORD PTR [rbp - 96], rax");                       // save the cast string for cleanup
    emitter.instruction("call __rt_hash_normalize_key");                       // rax=key_lo, rdx=key_hi (rax/rdx still hold ptr/len)
    emitter.instruction("mov QWORD PTR [rbp - 80], rax");                       // save the normalized key low word
    emitter.instruction("mov QWORD PTR [rbp - 88], rdx");                       // save the normalized key high word
    emitter.label("__rt_array_combine_mixed_key_ready");
    // -- insert the coerced key + owned value into the result hash --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 24]");                       // result hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // coerced key low word
    emitter.instruction("mov rdx, QWORD PTR [rbp - 88]");                       // coerced key high word
    emitter.instruction("mov rcx, QWORD PTR [rbp - 72]");                       // owned value cell
    emitter.instruction("xor r8, r8");                                          // boxed Mixed payloads use no high word
    emitter.instruction("mov r9, 7");                                           // value_type 7 = boxed Mixed
    emitter.instruction("call __rt_hash_set");                                  // insert or overwrite the pair (persists a string-key copy)
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the possibly-grown result hash
    // -- release the per-iteration key temporaries --
    emitter.instruction("mov rax, QWORD PTR [rbp - 96]");                       // cast-string temp (0 unless the key was string-cast)
    emitter.instruction("test rax, rax");                                       // skip when no cast string was allocated
    emitter.instruction("je __rt_array_combine_mixed_no_str");                  // no cast string to release
    emitter.instruction("call __rt_decref_any");                                // free the cast string (hash_set kept its own copy)
    emitter.label("__rt_array_combine_mixed_no_str");
    emitter.instruction("mov rax, QWORD PTR [rbp - 104]");                      // boxed-key temp (0 unless the key was reboxed)
    emitter.instruction("test rax, rax");                                       // skip when no boxed key was allocated
    emitter.instruction("je __rt_array_combine_mixed_no_kbox");                 // no boxed key to release
    emitter.instruction("call __rt_decref_mixed");                              // free the boxed key source
    emitter.label("__rt_array_combine_mixed_no_kbox");
    emitter.instruction("jmp __rt_array_combine_mixed_loop");                   // combine the next pair
    // -- done: deep-free the temp hashes and return the result --
    emitter.label("__rt_array_combine_mixed_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // keys hash
    emitter.instruction("call __rt_decref_hash");                               // deep-free the owned keys hash
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // values hash
    emitter.instruction("call __rt_decref_hash");                               // deep-free the owned values hash
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // rax = the combined result hash
    emitter.instruction("mov rsp, rbp");                                        // restore the stack pointer
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                // return the combined hash
    // -- count mismatch: free the temp hashes then throw a catchable ValueError --
    emitter.label("__rt_array_combine_mixed_mismatch");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // keys hash
    emitter.instruction("call __rt_decref_hash");                               // deep-free the owned keys hash before throwing
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // values hash
    emitter.instruction("call __rt_decref_hash");                               // deep-free the owned values hash before throwing
    value_error::emit_throw_value_error_x86_64(
        emitter,
        "_array_combine_count_msg",
        ARRAY_COMBINE_COUNT_MSG_LEN,
    );
}
