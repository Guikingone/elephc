//! Purpose:
//! Emits the associative-array runtime helper for PHP `array_change_key_case`.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` through the arrays runtime module.
//!
//! Key details:
//! - String keys are converted with PHP's byte-oriented ASCII case rules and rehashed.
//! - Integer keys and value ownership are preserved in a fresh COW-safe hash table.

use crate::codegen::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Byte length of the shared PHP array-argument TypeError diagnostic.
const ARRAY_ARG_TYPE_ERROR_MSG_LEN: usize = 78;

/// Emits `__rt_array_change_key_case_hash` for the active supported target.
///
/// Input is a borrowed source hash plus an integer mode (`0` for lowercase,
/// any other value for uppercase). The result is a fresh associative array in
/// insertion order; case collisions update the first matching slot with the
/// last source value, matching PHP.
pub fn emit_array_change_key_case(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_array_change_key_case_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: array_change_key_case hash ---");
    emitter.label_global("__rt_array_change_key_case_hash");

    // Stack layout:
    //   [sp, #0]  = source hash
    //   [sp, #8]  = destination hash
    //   [sp, #16] = case mode
    //   [sp, #24] = insertion-order cursor
    //   [sp, #32] = converted key low word
    //   [sp, #40] = converted key high word
    //   [sp, #48] = owned value low word
    //   [sp, #56] = owned value high word
    //   [sp, #64] = runtime value tag
    //   [sp, #80] = saved frame pointer and return address
    emitter.instruction("sub sp, sp, #96");                                     // reserve aligned spill storage for the source entry and converted result state
    emitter.instruction("stp x29, x30, [sp, #80]");                             // preserve the caller frame pointer and return address across nested runtime calls
    emitter.instruction("add x29, sp, #80");                                    // establish a stable frame base above the helper spill slots
    emitter.instruction("str x0, [sp, #0]");                                    // preserve the borrowed source associative-array pointer across allocation and iteration
    emitter.instruction("str x1, [sp, #16]");                                   // preserve the requested lowercase-or-uppercase mode across the full rebuild
    emitter.instruction("ldr x0, [x0, #8]");                                    // pass the source hash capacity to the destination allocator
    emitter.instruction("ldr x2, [sp, #0]");                                    // reload the source hash pointer after repurposing the first argument register
    emitter.instruction("ldr x1, [x2, #16]");                                   // pass the source table-wide runtime value tag to the destination allocator
    emitter.instruction("bl __rt_hash_new");                                    // allocate a fresh associative array with matching capacity and value layout
    emitter.instruction("str x0, [sp, #8]");                                    // preserve the current destination pointer across iteration and possible hash growth
    emitter.instruction("str xzr, [sp, #24]");                                  // initialize the insertion-order iterator cursor at the source hash head

    emitter.label("__rt_array_change_key_case_loop");
    emitter.instruction("ldr x0, [sp, #0]");                                    // pass the borrowed source associative array to the insertion-order iterator
    emitter.instruction("ldr x1, [sp, #24]");                                   // resume iteration from the cursor returned by the previous source entry
    emitter.instruction("bl __rt_hash_iter_next");                              // fetch the next source key, value payload, tag, and continuation cursor
    emitter.instruction("cmp x0, #-1");                                         // did the iterator report that every source entry has been consumed?
    emitter.instruction("b.eq __rt_array_change_key_case_done");                // return the rebuilt hash after the final source entry
    emitter.instruction("str x0, [sp, #24]");                                   // preserve the next insertion-order cursor across key and value helper calls
    emitter.instruction("str x1, [sp, #32]");                                   // preserve the current source key pointer or inline integer key
    emitter.instruction("str x2, [sp, #40]");                                   // preserve the current source key length or integer-key sentinel
    emitter.instruction("str x3, [sp, #48]");                                   // preserve the current source value low word before acquiring result ownership
    emitter.instruction("str x4, [sp, #56]");                                   // preserve the current source value high word before acquiring result ownership
    emitter.instruction("str x5, [sp, #64]");                                   // preserve the current source runtime value tag for ownership dispatch
    emitter.instruction("cmp x2, #-1");                                         // is this an inline integer key that must remain numerically unchanged?
    emitter.instruction("b.eq __rt_array_change_key_case_key_ready");           // skip string conversion for integer keys
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the borrowed source string-key pointer to the selected case helper
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass the borrowed source string-key length to the selected case helper
    emitter.instruction("ldr x9, [sp, #16]");                                   // load the requested key-case mode for lowercase-versus-uppercase dispatch
    emitter.instruction("cbnz x9, __rt_array_change_key_case_upper");           // PHP treats every nonzero mode value as uppercase
    emitter.instruction("bl __rt_strtolower");                                  // materialize an ASCII-lowercase transient copy of the source string key
    emitter.instruction("b __rt_array_change_key_case_key_store");              // skip the uppercase conversion path after producing the lowercase key
    emitter.label("__rt_array_change_key_case_upper");
    emitter.instruction("bl __rt_strtoupper");                                  // materialize an ASCII-uppercase transient copy of the source string key
    emitter.label("__rt_array_change_key_case_key_store");
    emitter.instruction("str x1, [sp, #32]");                                   // preserve the converted transient key pointer for hash insertion or collision lookup
    emitter.instruction("str x2, [sp, #40]");                                   // preserve the unchanged converted key byte length

    emitter.label("__rt_array_change_key_case_key_ready");
    emitter.instruction("ldr x9, [sp, #64]");                                   // load the current value tag before acquiring ownership for the fresh result hash
    emitter.instruction("cmp x9, #1");                                          // is the source value a string payload that needs an independent persisted copy?
    emitter.instruction("b.eq __rt_array_change_key_case_value_string");        // duplicate string payloads before the result hash takes ownership
    emitter.instruction("cmp x9, #4");                                          // is the source value a nested indexed array requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // retain nested indexed arrays for the new result owner
    emitter.instruction("cmp x9, #5");                                          // is the source value a nested associative array requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // retain nested associative arrays for the new result owner
    emitter.instruction("cmp x9, #6");                                          // is the source value an object requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // retain objects for the new result owner
    emitter.instruction("cmp x9, #7");                                          // is the source value a boxed Mixed cell requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // retain boxed Mixed cells for the new result owner
    emitter.instruction("cmp x9, #10");                                         // is the source value a callable descriptor requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // retain callable descriptors for the new result owner
    emitter.instruction("cmp x9, #11");                                         // is the source value a PHP reference cell requiring a retained reference?
    emitter.instruction("b.eq __rt_array_change_key_case_value_ref");           // preserve reference identity by retaining the shared cell
    emitter.instruction("ldr x3, [sp, #48]");                                   // reload the scalar or float low word that can be copied unchanged
    emitter.instruction("ldr x4, [sp, #56]");                                   // reload the scalar or float high word that can be copied unchanged
    emitter.instruction("ldr x5, [sp, #64]");                                   // reload the scalar or float runtime tag for the destination entry
    emitter.instruction("b __rt_array_change_key_case_insert");                 // insert the non-owning scalar payload into the rebuilt hash

    emitter.label("__rt_array_change_key_case_value_string");
    emitter.instruction("ldr x1, [sp, #48]");                                   // pass the borrowed source string-value pointer to the persistence helper
    emitter.instruction("ldr x2, [sp, #56]");                                   // pass the borrowed source string-value length to the persistence helper
    emitter.instruction("bl __rt_str_persist");                                 // allocate an independently owned string payload for the result hash
    emitter.instruction("mov x3, x1");                                          // stage the persisted string pointer as the destination value low word
    emitter.instruction("mov x4, x2");                                          // stage the persisted string length as the destination value high word
    emitter.instruction("ldr x5, [sp, #64]");                                   // reload the string runtime tag for the destination entry
    emitter.instruction("b __rt_array_change_key_case_insert");                 // insert the independently owned string value into the rebuilt hash

    emitter.label("__rt_array_change_key_case_value_ref");
    emitter.instruction("ldr x0, [sp, #48]");                                   // pass the shared source child pointer to the retain helper
    emitter.instruction("bl __rt_incref");                                      // acquire one reference for the fresh destination hash owner
    emitter.instruction("ldr x3, [sp, #48]");                                   // reload the retained child pointer as the destination value low word
    emitter.instruction("mov x4, xzr");                                         // clear the unused high word for refcounted destination values
    emitter.instruction("ldr x5, [sp, #64]");                                   // reload the refcounted runtime tag for the destination entry

    emitter.label("__rt_array_change_key_case_insert");
    emitter.instruction("ldr x0, [sp, #8]");                                    // pass the current destination hash pointer to insert-or-update
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the converted string key or unchanged inline integer key
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass the converted key length or integer-key sentinel
    emitter.instruction("bl __rt_hash_set");                                    // persist new string keys, resolve collisions, and consume the owned value payload
    emitter.instruction("str x0, [sp, #8]");                                    // preserve a possibly grown destination hash for the next entry and final result
    emitter.instruction("b __rt_array_change_key_case_loop");                   // continue rebuilding entries in original insertion order

    emitter.label("__rt_array_change_key_case_done");
    emitter.instruction("ldr x0, [sp, #8]");                                    // return the fresh rebuilt associative-array pointer
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore the caller frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the helper spill frame before returning
    emitter.instruction("ret");                                                 // return with the case-converted associative array in the integer result register

    // Mixed/union boundary: inspect the boxed PHP array representation, run the
    // matching clone path, and transfer the fresh result into a new Mixed box.
    emitter.blank();
    emitter.comment("--- runtime: array_change_key_case mixed dispatch ---");
    emitter.label_global("__rt_array_change_key_case_mixed");
    emitter.instruction("sub sp, sp, #48");                                     // reserve aligned storage for mode, runtime tag, raw result, and boxed result
    emitter.instruction("stp x29, x30, [sp, #32]");                             // preserve the caller frame pointer and return address across dispatch and boxing calls
    emitter.instruction("add x29, sp, #32");                                    // establish a stable frame base above the mixed-dispatch spill slots
    emitter.instruction("str x1, [sp, #0]");                                    // preserve the requested case mode while inspecting the boxed array payload
    emitter.instruction("ldr x9, [x0]");                                        // load the concrete runtime tag from the borrowed boxed Mixed cell
    emitter.instruction("str x9, [sp, #8]");                                    // preserve the concrete array tag for result boxing after the clone call
    emitter.instruction("ldr x0, [x0, #8]");                                    // unbox the borrowed indexed-array or associative-array payload pointer
    emitter.instruction("cbz x0, __rt_array_change_key_case_mixed_bad");        // reject null container payloads with the shared PHP array TypeError
    emitter.instruction("cmp x9, #4");                                          // does the boxed value contain a packed indexed array?
    emitter.instruction("b.eq __rt_array_change_key_case_mixed_indexed");       // integer-key arrays only need the existing COW-safe shallow clone
    emitter.instruction("cmp x9, #5");                                          // does the boxed value contain an associative hash?
    emitter.instruction("b.ne __rt_array_change_key_case_mixed_bad");           // any other boxed payload violates the PHP array parameter contract
    emitter.instruction("ldr x1, [sp, #0]");                                    // restore the requested case mode beside the unboxed associative-array pointer
    emitter.instruction("bl __rt_array_change_key_case_hash");                  // rebuild and rehash the associative array with converted string keys
    emitter.instruction("b __rt_array_change_key_case_mixed_box");              // box the freshly owned associative-array result
    emitter.label("__rt_array_change_key_case_mixed_indexed");
    emitter.instruction("bl __rt_array_clone_shallow");                         // clone the packed array because its integer keys are unaffected by case conversion
    emitter.label("__rt_array_change_key_case_mixed_box");
    emitter.instruction("str x0, [sp, #16]");                                   // preserve the freshly owned raw array result across Mixed-cell allocation
    emitter.instruction("mov x1, x0");                                          // pass the raw array pointer as the low payload word to the Mixed boxer
    emitter.instruction("ldr x0, [sp, #8]");                                    // pass runtime tag 4 or 5 so the Mixed box records the concrete array representation
    emitter.instruction("mov x2, xzr");                                         // clear the unused high payload word for array-backed Mixed values
    emitter.instruction("bl __rt_mixed_from_value");                            // allocate a Mixed result box and retain its freshly created array child
    emitter.instruction("str x0, [sp, #24]");                                   // preserve the boxed result while releasing the temporary raw-owner reference
    emitter.instruction("ldr x0, [sp, #16]");                                   // pass the raw fresh array owner to the kind-aware release helper
    emitter.instruction("bl __rt_decref_any");                                  // transfer sole child ownership into the returned Mixed box without leaking a reference
    emitter.instruction("ldr x0, [sp, #24]");                                   // restore the independently owned boxed Mixed result
    emitter.instruction("ldp x29, x30, [sp, #32]");                             // restore the caller frame pointer and return address after result boxing
    emitter.instruction("add sp, sp, #48");                                     // release the mixed-dispatch spill frame
    emitter.instruction("ret");                                                 // return the boxed case-converted array in the integer result register
    emitter.label("__rt_array_change_key_case_mixed_bad");
    abi::emit_symbol_address(emitter, "x1", "_array_arg_type_error_msg");
    emitter.instruction(&format!("mov x2, #{}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the shared array-argument TypeError message length
    emitter.instruction("mov x0, #2");                                          // select stderr for the fatal PHP argument diagnostic
    emitter.syscall(4);
    emitter.instruction("mov x0, #70");                                         // use EX_SOFTWARE after the fatal array argument violation
    emitter.syscall(1);
}

/// Emits the Linux x86_64 SysV variant of `__rt_array_change_key_case_hash`.
///
/// Input is `rdi = source hash`, `rsi = case mode`; output is a fresh hash in
/// `rax`. The implementation mirrors the ARM64 ownership and collision rules.
fn emit_array_change_key_case_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: array_change_key_case hash ---");
    emitter.label_global("__rt_array_change_key_case_hash");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving rebuild spill storage
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for source, destination, cursor, key, and value state
    emitter.instruction("sub rsp, 80");                                         // reserve aligned spill storage while keeping every nested SysV call correctly aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // preserve the borrowed source associative-array pointer across allocation and iteration
    emitter.instruction("mov QWORD PTR [rbp - 24], rsi");                       // preserve the requested lowercase-or-uppercase mode across the full rebuild
    emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                        // pass the source hash capacity to the destination allocator
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the source hash pointer after repurposing the first argument register
    emitter.instruction("mov rsi, QWORD PTR [rax + 16]");                       // pass the source table-wide runtime value tag to the destination allocator
    emitter.instruction("call __rt_hash_new");                                  // allocate a fresh associative array with matching capacity and value layout
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve the current destination pointer across iteration and possible hash growth
    emitter.instruction("mov QWORD PTR [rbp - 32], 0");                         // initialize the insertion-order iterator cursor at the source hash head

    emitter.label("__rt_array_change_key_case_loop");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the borrowed source associative array to the insertion-order iterator
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // resume iteration from the cursor returned by the previous source entry
    emitter.instruction("call __rt_hash_iter_next");                            // fetch the next source key, value payload, tag, and continuation cursor
    emitter.instruction("cmp rax, -1");                                         // did the iterator report that every source entry has been consumed?
    emitter.instruction("je __rt_array_change_key_case_done");                  // return the rebuilt hash after the final source entry
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the next insertion-order cursor across key and value helper calls
    emitter.instruction("mov QWORD PTR [rbp - 40], rdi");                       // preserve the current source key pointer or inline integer key
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // preserve the current source key length or integer-key sentinel
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                       // preserve the current source value low word before acquiring result ownership
    emitter.instruction("mov QWORD PTR [rbp - 64], r8");                        // preserve the current source value high word before acquiring result ownership
    emitter.instruction("mov QWORD PTR [rbp - 72], r9");                        // preserve the current source runtime value tag for ownership dispatch
    emitter.instruction("cmp rdx, -1");                                         // is this an inline integer key that must remain numerically unchanged?
    emitter.instruction("je __rt_array_change_key_case_key_ready");             // skip string conversion for integer keys
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // pass the borrowed source string-key pointer to the selected x86 string helper
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // pass the borrowed source string-key length to the selected x86 string helper
    emitter.instruction("cmp QWORD PTR [rbp - 24], 0");                         // does the caller request PHP's lowercase key mode?
    emitter.instruction("jne __rt_array_change_key_case_upper");                // PHP treats every nonzero mode value as uppercase
    emitter.instruction("call __rt_strtolower");                                // materialize an ASCII-lowercase transient copy of the source string key
    emitter.instruction("jmp __rt_array_change_key_case_key_store");            // skip the uppercase conversion path after producing the lowercase key
    emitter.label("__rt_array_change_key_case_upper");
    emitter.instruction("call __rt_strtoupper");                                // materialize an ASCII-uppercase transient copy of the source string key
    emitter.label("__rt_array_change_key_case_key_store");
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // preserve the converted transient key pointer for hash insertion or collision lookup
    emitter.instruction("mov QWORD PTR [rbp - 48], rdx");                       // preserve the unchanged converted key byte length

    emitter.label("__rt_array_change_key_case_key_ready");
    emitter.instruction("mov r10, QWORD PTR [rbp - 72]");                       // load the current value tag before acquiring ownership for the fresh result hash
    emitter.instruction("cmp r10, 1");                                          // is the source value a string payload that needs an independent persisted copy?
    emitter.instruction("je __rt_array_change_key_case_value_string");          // duplicate string payloads before the result hash takes ownership
    emitter.instruction("cmp r10, 4");                                          // is the source value a nested indexed array requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // retain nested indexed arrays for the new result owner
    emitter.instruction("cmp r10, 5");                                          // is the source value a nested associative array requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // retain nested associative arrays for the new result owner
    emitter.instruction("cmp r10, 6");                                          // is the source value an object requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // retain objects for the new result owner
    emitter.instruction("cmp r10, 7");                                          // is the source value a boxed Mixed cell requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // retain boxed Mixed cells for the new result owner
    emitter.instruction("cmp r10, 10");                                         // is the source value a callable descriptor requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // retain callable descriptors for the new result owner
    emitter.instruction("cmp r10, 11");                                         // is the source value a PHP reference cell requiring a retained reference?
    emitter.instruction("je __rt_array_change_key_case_value_ref");             // preserve reference identity by retaining the shared cell
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // reload the scalar or float low word that can be copied unchanged
    emitter.instruction("mov r8, QWORD PTR [rbp - 64]");                        // reload the scalar or float high word that can be copied unchanged
    emitter.instruction("mov r9, QWORD PTR [rbp - 72]");                        // reload the scalar or float runtime tag for the destination entry
    emitter.instruction("jmp __rt_array_change_key_case_insert");               // insert the non-owning scalar payload into the rebuilt hash

    emitter.label("__rt_array_change_key_case_value_string");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // pass the borrowed source string-value pointer to the persistence helper
    emitter.instruction("mov rdx, QWORD PTR [rbp - 64]");                       // pass the borrowed source string-value length to the persistence helper
    emitter.instruction("call __rt_str_persist");                               // allocate an independently owned string payload for the result hash
    emitter.instruction("mov rcx, rax");                                        // stage the persisted string pointer as the destination value low word
    emitter.instruction("mov r8, rdx");                                         // stage the persisted string length as the destination value high word
    emitter.instruction("mov r9, QWORD PTR [rbp - 72]");                        // reload the string runtime tag for the destination entry
    emitter.instruction("jmp __rt_array_change_key_case_insert");               // insert the independently owned string value into the rebuilt hash

    emitter.label("__rt_array_change_key_case_value_ref");
    emitter.instruction("mov rax, QWORD PTR [rbp - 56]");                       // pass the shared source child pointer to the retain helper
    emitter.instruction("call __rt_incref");                                    // acquire one reference for the fresh destination hash owner
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // reload the retained child pointer as the destination value low word
    emitter.instruction("xor r8d, r8d");                                        // clear the unused high word for refcounted destination values
    emitter.instruction("mov r9, QWORD PTR [rbp - 72]");                        // reload the refcounted runtime tag for the destination entry

    emitter.label("__rt_array_change_key_case_insert");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // pass the current destination hash pointer to insert-or-update
    emitter.instruction("mov rsi, QWORD PTR [rbp - 40]");                       // pass the converted string key or unchanged inline integer key
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // pass the converted key length or integer-key sentinel
    emitter.instruction("call __rt_hash_set");                                  // persist new string keys, resolve collisions, and consume the owned value payload
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve a possibly grown destination hash for the next entry and final result
    emitter.instruction("jmp __rt_array_change_key_case_loop");                 // continue rebuilding entries in original insertion order

    emitter.label("__rt_array_change_key_case_done");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the fresh rebuilt associative-array pointer
    emitter.instruction("add rsp, 80");                                         // release the aligned helper spill frame before returning
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer after the rebuild completes
    emitter.instruction("ret");                                                 // return with the case-converted associative array in the integer result register

    emitter.blank();
    emitter.comment("--- runtime: array_change_key_case mixed dispatch ---");
    emitter.label_global("__rt_array_change_key_case_mixed");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer before reserving mixed-dispatch spill storage
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for mode, tag, raw result, and boxed result state
    emitter.instruction("sub rsp, 48");                                         // reserve aligned spill storage while keeping nested SysV calls correctly aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // preserve the requested case mode while inspecting the boxed array payload
    emitter.instruction("mov rax, QWORD PTR [rdi]");                            // load the concrete runtime tag from the borrowed boxed Mixed cell
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // preserve the concrete array tag for result boxing after the clone call
    emitter.instruction("mov rdi, QWORD PTR [rdi + 8]");                        // unbox the borrowed indexed-array or associative-array payload pointer
    emitter.instruction("test rdi, rdi");                                       // is the boxed array payload pointer non-null?
    emitter.instruction("jz __rt_array_change_key_case_mixed_bad");             // reject null container payloads with the shared PHP array TypeError
    emitter.instruction("cmp rax, 4");                                          // does the boxed value contain a packed indexed array?
    emitter.instruction("je __rt_array_change_key_case_mixed_indexed");         // integer-key arrays only need the existing COW-safe shallow clone
    emitter.instruction("cmp rax, 5");                                          // does the boxed value contain an associative hash?
    emitter.instruction("jne __rt_array_change_key_case_mixed_bad");            // any other boxed payload violates the PHP array parameter contract
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // restore the requested case mode beside the unboxed associative-array pointer
    emitter.instruction("call __rt_array_change_key_case_hash");                // rebuild and rehash the associative array with converted string keys
    emitter.instruction("jmp __rt_array_change_key_case_mixed_box");            // box the freshly owned associative-array result
    emitter.label("__rt_array_change_key_case_mixed_indexed");
    emitter.instruction("call __rt_array_clone_shallow");                       // clone the packed array because its integer keys are unaffected by case conversion
    emitter.label("__rt_array_change_key_case_mixed_box");
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // preserve the freshly owned raw array result across Mixed-cell allocation
    emitter.instruction("mov rdi, rax");                                        // pass the raw array pointer as the low payload word to the Mixed boxer
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // pass runtime tag 4 or 5 so the Mixed box records the concrete array representation
    emitter.instruction("xor esi, esi");                                        // clear the unused high payload word for array-backed Mixed values
    emitter.instruction("call __rt_mixed_from_value");                          // allocate a Mixed result box and retain its freshly created array child
    emitter.instruction("mov QWORD PTR [rbp - 32], rax");                       // preserve the boxed result while releasing the temporary raw-owner reference
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // pass the raw fresh array owner to the kind-aware release helper
    emitter.instruction("call __rt_decref_any");                                // transfer sole child ownership into the returned Mixed box without leaking a reference
    emitter.instruction("mov rax, QWORD PTR [rbp - 32]");                       // restore the independently owned boxed Mixed result
    emitter.instruction("add rsp, 48");                                         // release the mixed-dispatch spill frame after result boxing
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the boxed result
    emitter.instruction("ret");                                                 // return the boxed case-converted array in the integer result register
    emitter.label("__rt_array_change_key_case_mixed_bad");
    emitter.instruction("mov edi, 2");                                          // select stderr for the fatal PHP argument diagnostic
    abi::emit_symbol_address(emitter, "rsi", "_array_arg_type_error_msg");
    emitter.instruction(&format!("mov edx, {}", ARRAY_ARG_TYPE_ERROR_MSG_LEN)); // pass the shared array-argument TypeError message length
    emitter.instruction("mov eax, 1");                                          // select Linux write for the fatal PHP argument diagnostic
    emitter.instruction("syscall");                                             // emit the shared array-argument TypeError message
    emitter.instruction("mov edi, 70");                                         // use EX_SOFTWARE after the fatal array argument violation
    emitter.instruction("mov eax, 60");                                         // select Linux exit after emitting the fatal diagnostic
    emitter.instruction("syscall");                                             // terminate the process after the PHP array argument violation
}

#[cfg(test)]
mod tests {
    use crate::codegen_support::emit::Emitter;
    use crate::codegen_support::platform::{Arch, Platform, Target};

    use super::*;

    /// Verifies all supported targets emit both the concrete-hash helper and the
    /// boxed-Mixed representation dispatcher with their target-specific calls.
    #[test]
    fn test_array_change_key_case_emits_all_supported_target_paths() {
        for target in [
            Target::new(Platform::MacOS, Arch::AArch64),
            Target::new(Platform::Linux, Arch::AArch64),
            Target::new(Platform::Linux, Arch::X86_64),
        ] {
            let mut emitter = Emitter::new(target);
            emit_array_change_key_case(&mut emitter);
            let asm = emitter.output();

            assert!(asm.contains("__rt_array_change_key_case_hash:"));
            assert!(asm.contains("__rt_array_change_key_case_mixed:"));
            assert!(asm.contains("__rt_strtolower"));
            assert!(asm.contains("__rt_strtoupper"));
            assert!(asm.contains("__rt_hash_set"));
            assert!(asm.contains("__rt_array_clone_shallow"));
            assert!(asm.contains("__rt_mixed_from_value"));
        }
    }
}
