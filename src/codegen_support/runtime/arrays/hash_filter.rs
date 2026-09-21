//! Purpose:
//! Emits the `__rt_hash_filter` runtime helper: `array_filter()` over an ASSOCIATIVE source
//! (a hash), producing a hash that carries the SOURCE keys of the entries the callback kept.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::arrays`.
//!
//! Key details:
//! - The indexed counterpart is `array_filter.rs`, which builds a LIST. php never renumbers a
//!   filtered array — it drops entries and keeps every surviving key — so an associative source
//!   needs a hash destination, which is this module. (The indexed helper renumbering is a
//!   separate, older divergence from php; see `.plans/generics.md`.)
//! - The walk and the callback argument ABI are `__rt_hash_map`'s: `__rt_hash_iter_next` in
//!   insertion order, and the per-entry RUNTIME value tag chooses between the two-register
//!   string ABI (tag 1) and one scalar register (everything else).
//! - Only `ARRAY_FILTER_USE_VALUE` reaches here. The other two modes pass a KEY, whose shape is
//!   independent of the value's, so their argument ABI is the product of both shapes; the
//!   lowering refuses them for a hash source with a message that says so rather than guessing.
//! - OWNERSHIP is where this differs from `__rt_hash_map`, and it is the whole risk of the file.
//!   A map inserts the CALLBACK's result, which the wrapper already transferred. A filter
//!   inserts the SOURCE's own value, which the source still owns, so every surviving value is
//!   retained for the destination first — `__rt_str_persist` duplicates a string, `__rt_incref`
//!   retains a container, a boxed Mixed cell or a callable descriptor, and a scalar needs
//!   nothing. That per-tag ladder is `__rt_hash_clone_shallow`'s, for the same reason.
//!   KEYS are not retained here: `__rt_hash_set` persists an inserted string key itself.
//! - The destination comes from `__rt_hash_new`, so it can never alias the source.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_hash_filter` runtime helper for associative (hash) sources.
///
/// Walks the source hash in insertion order, invokes the callback once per entry with that
/// entry's VALUE, and inserts `source key => source value` into a freshly allocated destination
/// hash for every entry whose callback returned a non-zero result.
///
/// # ABI
/// - Input: `x0` / `rdi` = callback function pointer, `x1` / `rsi` = source hash pointer,
///   `x2` / `rdx` = callback environment pointer (`0` when the callback captures nothing).
/// - Output: `x0` / `rax` = destination hash pointer.
pub fn emit_hash_filter(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_hash_filter_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: hash_filter ---");
    emitter.label_global("__rt_hash_filter");

    // Stack layout:
    //   [sp, #0]  = insertion-order iterator cursor
    //   [sp, #8]  = source key_lo (pointer, or inline integer key payload)
    //   [sp, #16] = source key_hi (-1 marks an inline integer key)
    //   [sp, #24] = source value_lo
    //   [sp, #32] = source value_hi
    //   [sp, #40] = source value_tag
    //   [sp, #64] = saved x21/x22
    //   [sp, #80] = saved x19/x20
    //   [sp, #96] = saved x29/x30
    emitter.instruction("sub sp, sp, #112");                                    // allocate the hash-filter frame
    emitter.instruction("stp x29, x30, [sp, #96]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #96");                                    // set up the hash-filter frame pointer
    emitter.instruction("stp x19, x20, [sp, #80]");                             // save callee-saved x19/x20 for the source and destination tables
    emitter.instruction("stp x21, x22, [sp, #64]");                             // save callee-saved x21/x22 for the callback and its environment
    emitter.instruction("mov x21, x0");                                         // x21 = callback address, live across every loop iteration
    emitter.instruction("mov x19, x1");                                         // x19 = source hash pointer, live across every helper call
    emitter.instruction("mov x22, x2");                                         // x22 = callback environment pointer (0 when unused)

    // -- allocate the destination with the SOURCE's value_type, since it stores source values --
    emitter.instruction("ldr x0, [x19]");                                       // x0 = source entry count
    emitter.instruction("lsl x0, x0, #1");                                      // double it to give the destination insertion headroom
    emitter.instruction("mov x9, #16");                                         // x9 = minimum destination bucket count
    emitter.instruction("cmp x0, x9");                                          // compare the derived capacity against the runtime minimum
    emitter.instruction("csel x0, x9, x0, lt");                                 // clamp very small sources up to the minimum bucket count
    emitter.instruction("ldr x1, [x19, #16]");                                  // x1 = source runtime value_type tag; the kept values keep their shape
    emitter.instruction("bl __rt_hash_new");                                    // allocate the destination hash
    emitter.instruction("mov x20, x0");                                         // x20 = destination hash pointer, updated after every insertion
    emitter.instruction("str xzr, [sp, #0]");                                   // iterator cursor = 0 (start from header.head)

    // -- walk the source hash in insertion order --
    emitter.label("__rt_hash_filter_loop");
    emitter.instruction("mov x0, x19");                                         // x0 = source hash pointer
    emitter.instruction("ldr x1, [sp, #0]");                                    // x1 = current insertion-order cursor
    emitter.instruction("bl __rt_hash_iter_next");                              // fetch the next source entry
    emitter.instruction("cmn x0, #1");                                          // did the iterator signal end-of-walk?
    emitter.instruction("b.eq __rt_hash_filter_done");                          // yes - the destination hash is complete
    emitter.instruction("str x0, [sp, #0]");                                    // save the next insertion-order cursor
    emitter.instruction("str x1, [sp, #8]");                                    // save the source key_lo across the callback call
    emitter.instruction("str x2, [sp, #16]");                                   // save the source key_hi across the callback call
    emitter.instruction("str x3, [sp, #24]");                                   // save the source value_lo; the destination stores it on a keep
    emitter.instruction("str x4, [sp, #32]");                                   // save the source value_hi
    emitter.instruction("str x5, [sp, #40]");                                   // save the source value_tag, which also picks the callback ABI

    // -- php passes the VALUE only; the argument ABI follows the entry's runtime value tag --
    emitter.instruction("cmp x5, #1");                                          // runtime tag 1 = string, which uses the two-register string ABI
    emitter.instruction("b.eq __rt_hash_filter_call_str");                      // string values are passed as a pointer/length pair
    emitter.instruction("mov x0, x3");                                          // x0 = scalar source value (int, bool, or boxed Mixed pointer)
    emitter.instruction("mov x1, x22");                                         // pass the capture environment after the scalar argument
    emitter.instruction("b __rt_hash_filter_call");                             // invoke the callback through the shared call site

    emitter.label("__rt_hash_filter_call_str");
    emitter.instruction("mov x0, x3");                                          // x0 = source string pointer
    emitter.instruction("mov x1, x4");                                          // x1 = source string length
    emitter.instruction("mov x2, x22");                                         // pass the capture environment after the string pointer/length pair

    emitter.label("__rt_hash_filter_call");
    emitter.instruction("blr x21");                                             // invoke the user predicate on this entry's value
    emitter.instruction("cbz x0, __rt_hash_filter_loop");                       // a zero result drops the entry; nothing was retained, nothing leaks

    // -- kept: retain the SOURCE value for the destination, per this entry's runtime tag --
    emitter.instruction("ldr x5, [sp, #40]");                                   // x5 = source entry value_tag
    emitter.instruction("cmp x5, #1");                                          // is this entry's value a string?
    emitter.instruction("b.eq __rt_hash_filter_value_str");                     // string values need a fresh persisted payload
    emitter.instruction("cmp x5, #4");                                          // is this entry's value an indexed array?
    emitter.instruction("b.eq __rt_hash_filter_value_ref");                     // nested refcounted values need retains
    emitter.instruction("cmp x5, #5");                                          // is this entry's value an associative array?
    emitter.instruction("b.eq __rt_hash_filter_value_ref");                     // nested refcounted values need retains
    emitter.instruction("cmp x5, #6");                                          // is this entry's value an object?
    emitter.instruction("b.eq __rt_hash_filter_value_ref");                     // nested refcounted values need retains
    emitter.instruction("cmp x5, #7");                                          // is this entry's value a boxed mixed cell?
    emitter.instruction("b.eq __rt_hash_filter_value_ref");                     // nested refcounted values need retains
    emitter.instruction("cmp x5, #10");                                         // is this entry's value a callable descriptor?
    emitter.instruction("b.eq __rt_hash_filter_value_ref");                     // runtime descriptors need retains; static ones are ignored by incref
    emitter.instruction("ldr x3, [sp, #24]");                                   // x3 = scalar/float value_lo copied as-is
    emitter.instruction("ldr x4, [sp, #32]");                                   // x4 = scalar/float value_hi copied as-is
    emitter.instruction("b __rt_hash_filter_insert");                           // scalars own nothing and are ready to insert

    emitter.label("__rt_hash_filter_value_str");
    emitter.instruction("ldr x1, [sp, #24]");                                   // x1 = source string value pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // x2 = source string value length
    emitter.instruction("bl __rt_str_persist");                                 // duplicate the string so the destination owns its own bytes
    emitter.instruction("mov x3, x1");                                          // x3 = owned string value pointer
    emitter.instruction("mov x4, x2");                                          // x4 = owned string value length
    emitter.instruction("b __rt_hash_filter_insert");                           // insert the duplicated string value

    emitter.label("__rt_hash_filter_value_ref");
    emitter.instruction("ldr x0, [sp, #24]");                                   // x0 = shared refcounted child pointer
    emitter.instruction("bl __rt_incref");                                      // retain it for the destination table
    emitter.instruction("ldr x3, [sp, #24]");                                   // reload the retained child pointer after the helper call
    emitter.instruction("mov x4, xzr");                                         // refcounted hash values store only value_lo

    // -- the destination keeps the SOURCE key; hash_set persists a string key itself --
    emitter.label("__rt_hash_filter_insert");
    emitter.instruction("ldr x5, [sp, #40]");                                   // x5 = the entry's value_tag, carried across unchanged
    emitter.instruction("mov x0, x20");                                         // x0 = destination hash pointer
    emitter.instruction("ldr x1, [sp, #8]");                                    // x1 = source key_lo
    emitter.instruction("ldr x2, [sp, #16]");                                   // x2 = source key_hi (-1 marks an integer key)
    emitter.instruction("bl __rt_hash_set");                                    // insert the surviving pair under its original key
    emitter.instruction("mov x20, x0");                                         // keep the destination pointer current after possible growth
    emitter.instruction("b __rt_hash_filter_loop");                             // continue with the next source entry

    emitter.label("__rt_hash_filter_done");
    emitter.instruction("mov x0, x20");                                         // return the destination hash pointer
    emitter.instruction("ldp x21, x22, [sp, #64]");                             // restore callee-saved x21/x22
    emitter.instruction("ldp x19, x20, [sp, #80]");                             // restore callee-saved x19/x20
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #112");                                    // deallocate the hash-filter frame
    emitter.instruction("ret");                                                 // return with x0 = destination hash pointer
}

/// Emits the x86_64 System V variant of `__rt_hash_filter`.
///
/// Mirrors the AArch64 logic exactly; only the register convention differs.
///
/// # ABI notes
/// - `__rt_hash_iter_next` returns `rax`=cursor, `rdi`=key_lo, `rdx`=key_hi, `rcx`=value_lo,
///   `r8`=value_hi, `r9`=value_tag. `rdi` doubles as argument zero, so every returned field is
///   spilled before the callback call.
/// - `__rt_str_persist` reads and returns its pair in `rax`/`rdx` on this target, which is how
///   `__rt_hash_set` and `__rt_hash_map` call it too.
/// - `__rt_incref` takes and preserves its pointer in `rax`, not `rdi`.
/// - The frame is `push rbp` + `sub rsp, 96`: entry leaves `rsp ≡ 8 (mod 16)`, the push makes it
///   `≡ 0`, and 96 is a multiple of 16, so every nested `call` stays System V aligned.
fn emit_hash_filter_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: hash_filter ---");
    emitter.label_global("__rt_hash_filter");

    // Frame layout:
    //   [rbp - 8]  = source hash pointer
    //   [rbp - 16] = destination hash pointer
    //   [rbp - 24] = insertion-order iterator cursor
    //   [rbp - 32] = source key_lo
    //   [rbp - 40] = source key_hi
    //   [rbp - 48] = source value_lo
    //   [rbp - 56] = source value_hi
    //   [rbp - 64] = source value_tag
    //   [rbp - 72] = callback address
    //   [rbp - 80] = callback environment pointer
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the hash-filter frame
    emitter.instruction("sub rsp, 96");                                         // reserve the spill slots, keeping System V alignment
    emitter.instruction("mov QWORD PTR [rbp - 72], rdi");                       // save the callback address for every iteration
    emitter.instruction("mov QWORD PTR [rbp - 8], rsi");                        // save the source hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 80], rdx");                       // save the callback environment pointer

    // -- allocate the destination with the SOURCE's value_type, since it stores source values --
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // rax = source hash pointer
    emitter.instruction("mov rdi, QWORD PTR [rax]");                            // rdi = source entry count
    emitter.instruction("shl rdi, 1");                                          // double it to give the destination insertion headroom
    emitter.instruction("cmp rdi, 16");                                         // compare the derived capacity against the runtime minimum
    emitter.instruction("jge __rt_hash_filter_cap_ready_x86");                  // keep the derived capacity when it clears the minimum
    emitter.instruction("mov rdi, 16");                                         // clamp very small sources up to the minimum bucket count
    emitter.label("__rt_hash_filter_cap_ready_x86");
    emitter.instruction("mov rsi, QWORD PTR [rax + 16]");                       // rsi = source runtime value_type tag
    emitter.instruction("call __rt_hash_new");                                  // allocate the destination hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the destination hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // iterator cursor = 0 (start from header.head)

    // -- walk the source hash in insertion order --
    emitter.label("__rt_hash_filter_loop_x86");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // rdi = source hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 24]");                       // rsi = current insertion-order cursor
    emitter.instruction("call __rt_hash_iter_next");                            // rax=cursor, rdi=key_lo, rdx=key_hi, rcx=lo, r8=hi, r9=tag
    emitter.instruction("cmp rax, -1");                                         // did the iterator signal end-of-walk?
    emitter.instruction("je __rt_hash_filter_done_x86");                        // yes - the destination hash is complete
    emitter.instruction("mov QWORD PTR [rbp - 24], rax");                       // save the next insertion-order cursor
    emitter.instruction("mov QWORD PTR [rbp - 32], rdi");                       // spill key_lo before rdi is reused as argument zero
    emitter.instruction("mov QWORD PTR [rbp - 40], rdx");                       // spill key_hi before the callback call
    emitter.instruction("mov QWORD PTR [rbp - 48], rcx");                       // spill value_lo; the destination stores it on a keep
    emitter.instruction("mov QWORD PTR [rbp - 56], r8");                        // spill value_hi
    emitter.instruction("mov QWORD PTR [rbp - 64], r9");                        // spill value_tag, which also picks the callback ABI

    // -- php passes the VALUE only; the argument ABI follows the entry's runtime value tag --
    emitter.instruction("cmp r9, 1");                                           // runtime tag 1 = string, which uses the two-register string ABI
    emitter.instruction("je __rt_hash_filter_call_str_x86");                    // string values are passed as a pointer/length pair
    emitter.instruction("mov rdi, rcx");                                        // rdi = scalar source value (int, bool, or boxed Mixed pointer)
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // pass the capture environment after the scalar argument
    emitter.instruction("jmp __rt_hash_filter_call_x86");                       // invoke the callback through the shared call site

    emitter.label("__rt_hash_filter_call_str_x86");
    emitter.instruction("mov rdi, rcx");                                        // rdi = source string pointer
    emitter.instruction("mov rsi, r8");                                         // rsi = source string length
    emitter.instruction("mov rdx, QWORD PTR [rbp - 80]");                       // pass the capture environment after the pointer/length pair

    emitter.label("__rt_hash_filter_call_x86");
    emitter.instruction("call QWORD PTR [rbp - 72]");                           // invoke the user predicate on this entry's value
    emitter.instruction("test rax, rax");                                       // did the predicate keep this entry?
    emitter.instruction("jz __rt_hash_filter_loop_x86");                        // a zero result drops it; nothing was retained, nothing leaks

    // -- kept: retain the SOURCE value for the destination, per this entry's runtime tag --
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // r9 = source entry value_tag
    emitter.instruction("cmp r9, 1");                                           // is this entry's value a string?
    emitter.instruction("je __rt_hash_filter_value_str_x86");                   // string values need a fresh persisted payload
    emitter.instruction("cmp r9, 4");                                           // is this entry's value an indexed array?
    emitter.instruction("je __rt_hash_filter_value_ref_x86");                   // nested refcounted values need retains
    emitter.instruction("cmp r9, 5");                                           // is this entry's value an associative array?
    emitter.instruction("je __rt_hash_filter_value_ref_x86");                   // nested refcounted values need retains
    emitter.instruction("cmp r9, 6");                                           // is this entry's value an object?
    emitter.instruction("je __rt_hash_filter_value_ref_x86");                   // nested refcounted values need retains
    emitter.instruction("cmp r9, 7");                                           // is this entry's value a boxed mixed cell?
    emitter.instruction("je __rt_hash_filter_value_ref_x86");                   // nested refcounted values need retains
    emitter.instruction("cmp r9, 10");                                          // is this entry's value a callable descriptor?
    emitter.instruction("je __rt_hash_filter_value_ref_x86");                   // runtime descriptors need retains
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // rcx = scalar/float value_lo copied as-is
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // r8 = scalar/float value_hi copied as-is
    emitter.instruction("jmp __rt_hash_filter_insert_x86");                     // scalars own nothing and are ready to insert

    emitter.label("__rt_hash_filter_value_str_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // rax = source string value pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 56]");                       // rdx = source string value length
    emitter.instruction("call __rt_str_persist");                               // duplicate the string so the destination owns its own bytes
    emitter.instruction("mov rcx, rax");                                        // rcx = owned string value pointer
    emitter.instruction("mov r8, rdx");                                         // r8 = owned string value length
    emitter.instruction("jmp __rt_hash_filter_insert_x86");                     // insert the duplicated string value

    emitter.label("__rt_hash_filter_value_ref_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 48]");                       // rax = shared refcounted child pointer (incref reads rax here)
    emitter.instruction("call __rt_incref");                                    // retain it for the destination table
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // reload the retained child pointer after the helper call
    emitter.instruction("xor r8d, r8d");                                        // refcounted hash values store only value_lo

    // -- the destination keeps the SOURCE key; hash_set persists a string key itself --
    emitter.label("__rt_hash_filter_insert_x86");
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // r9 = the entry's value_tag, carried across unchanged
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // rdi = destination hash pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 32]");                       // rsi = source key_lo
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // rdx = source key_hi (-1 marks an integer key)
    emitter.instruction("call __rt_hash_set");                                  // insert the surviving pair under its original key
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // keep the destination pointer current after possible growth
    emitter.instruction("jmp __rt_hash_filter_loop_x86");                       // continue with the next source entry

    emitter.label("__rt_hash_filter_done_x86");
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // return the destination hash pointer
    emitter.instruction("mov rsp, rbp");                                        // discard the hash-filter frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return with rax = destination hash pointer
}
