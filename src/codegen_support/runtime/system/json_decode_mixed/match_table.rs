//! Purpose:
//! Emits the x86_64 bracket-match pre-pass that lets the structural JSON decoder skip
//! already-matched nested containers instead of rescanning them once per nesting level.
//!
//! Called from:
//! - `crate::codegen_support::runtime::system::json_decode_mixed::emit_json_decode_mixed`.
//!
//! Key details:
//! - The recursive decoder finds each element's end with a depth-and-string-aware scan, then
//!   decodes that sub-slice, whose own scan walks the same bytes again: a document nested `d`
//!   levels deep costs O(len * d). The pre-pass records, for every opening bracket outside a
//!   string, the offset of the bracket that closes it, using exactly the scanner's rules
//!   (`[`/`{` open, `]`/`}` close regardless of kind, `"` toggles strings, `\` escapes one byte).
//! - Element scanners start only at positions the top-level walk reaches outside a string, so
//!   the pre-pass's string state agrees with every scanner's. A scanner only uses a recorded
//!   match that lies strictly inside its own slice, which is exactly where its own depth walk
//!   would have found the same close, so decoding results and errors are unchanged.
//! - The table is built only for documents with nesting depth of at least 16 and length below
//!   1 MiB; shallower documents keep the plain scan, whose cost is already linear in practice.
//! - The table is an optimization, never a new way to run out of memory: it is only allocated
//!   when both the arena's unbumped tail and the live-memory budget have at least four times its
//!   size free, so the decode itself keeps the room it had without the table.
//! - The published pointer is cleared on entry, so a table left behind by a decode that never
//!   returned cannot be consulted, and a scanner only jumps forward past its own cursor.
//! - Layout: `[table]` = document base pointer, `[table + 8]` = document length, then one `u32`
//!   per document byte at `[table + 16]`. Unmatched opens hold `0xFFFFFFFF`.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;

/// Minimum document length before the depth probe runs at all.
const MIN_LEN: usize = 256;
/// Largest document length the table is built for (keeps the table at most 4 MiB).
const MAX_LEN: usize = 0x10_0000;
/// Minimum nesting depth for which the quadratic rescans outweigh building the table.
const MIN_DEPTH: usize = 16;

/// Emits `__rt_json_match_build` (rax = document pointer, rdx = document length).
/// Publishes `_json_match_table` when the document qualifies; preserves rax and rdx.
pub(super) fn emit_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: json_match_build ---");
    emitter.label_global("__rt_json_match_build");
    emitter.instruction("push rbp");                                            // save the decoder boundary frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the pre-pass frame
    emitter.instruction("sub rsp, 16");                                         // keep the allocation call aligned
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // preserve the document pointer for the decoder
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // preserve the document length for the decoder
    abi::emit_store_zero_to_symbol(emitter, "_json_match_table", 0);
    emitter.instruction(&format!("cmp rdx, {}", MIN_LEN));                      // short documents never pay for deep rescans
    emitter.instruction("jb __rt_json_match_build_done_x");                     // keep the plain scan for short input
    emitter.instruction(&format!("cmp rdx, {}", MAX_LEN));                      // bound the table to a modest heap footprint
    emitter.instruction("ja __rt_json_match_build_done_x");                     // keep the plain scan for very large input

    // -- pass 1: measure the maximum bracket depth with the scanner's string rules --
    emitter.instruction("xor ecx, ecx");                                        // byte index
    emitter.instruction("xor r8d, r8d");                                        // current depth
    emitter.instruction("xor r9d, r9d");                                        // maximum depth
    emitter.instruction("xor r10d, r10d");                                      // inside-string flag
    emitter.instruction("xor r11d, r11d");                                      // pending-escape flag
    emitter.label("__rt_json_match_depth_loop_x");
    emitter.instruction("cmp rcx, rdx");                                        // every document byte visited?
    emitter.instruction("jae __rt_json_match_depth_done_x");                    // the depth probe is complete
    emitter.instruction("movzx esi, BYTE PTR [rax + rcx]");                     // load the current byte
    emitter.instruction("test r11d, r11d");                                     // is this byte escaped?
    emitter.instruction("jne __rt_json_match_depth_unescape_x");                // an escaped byte is consumed verbatim
    emitter.instruction("test r10d, r10d");                                     // inside a string?
    emitter.instruction("jne __rt_json_match_depth_string_x");                  // only quotes and escapes matter there
    emitter.instruction("cmp esi, 34");                                         // '"'
    emitter.instruction("je __rt_json_match_depth_enter_string_x");             // a string starts
    emitter.instruction("cmp esi, 91");                                         // '['
    emitter.instruction("je __rt_json_match_depth_open_x");                     // an array opens
    emitter.instruction("cmp esi, 123");                                        // '{'
    emitter.instruction("je __rt_json_match_depth_open_x");                     // an object opens
    emitter.instruction("cmp esi, 93");                                         // ']'
    emitter.instruction("je __rt_json_match_depth_close_x");                    // a container closes
    emitter.instruction("cmp esi, 125");                                        // '}'
    emitter.instruction("je __rt_json_match_depth_close_x");                    // a container closes
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // any other byte leaves the depth alone
    emitter.label("__rt_json_match_depth_open_x");
    emitter.instruction("add r8, 1");                                           // one level deeper
    emitter.instruction("cmp r8, r9");                                          // new maximum?
    emitter.instruction("jbe __rt_json_match_depth_next_x");                    // no new maximum
    emitter.instruction("mov r9, r8");                                          // record the new maximum depth
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_close_x");
    emitter.instruction("test r8, r8");                                         // a stray close at depth zero?
    emitter.instruction("je __rt_json_match_depth_next_x");                     // stray closes do not change the depth
    emitter.instruction("sub r8, 1");                                           // one level shallower
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_enter_string_x");
    emitter.instruction("mov r10d, 1");                                         // the scan is now inside a string
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_string_x");
    emitter.instruction("cmp esi, 92");                                         // '\\'
    emitter.instruction("je __rt_json_match_depth_escape_x");                   // the next byte is escaped
    emitter.instruction("cmp esi, 34");                                         // '"'
    emitter.instruction("jne __rt_json_match_depth_next_x");                    // ordinary string byte
    emitter.instruction("xor r10d, r10d");                                      // the string ends
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_escape_x");
    emitter.instruction("mov r11d, 1");                                         // remember the pending escape
    emitter.instruction("jmp __rt_json_match_depth_next_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_unescape_x");
    emitter.instruction("xor r11d, r11d");                                      // the escaped byte is consumed
    emitter.label("__rt_json_match_depth_next_x");
    emitter.instruction("add rcx, 1");                                          // advance to the next byte
    emitter.instruction("jmp __rt_json_match_depth_loop_x");                    // continue the probe
    emitter.label("__rt_json_match_depth_done_x");
    emitter.instruction(&format!("cmp r9, {}", MIN_DEPTH));                     // deep enough for rescans to dominate?
    emitter.instruction("jb __rt_json_match_build_done_x");                     // shallow documents keep the plain scan

    // -- allocate only with ample free room, so the table cannot exhaust a working decode --
    emitter.instruction("lea rsi, [rdx * 4 + 16]");                             // table size in bytes
    emitter.instruction("shl rsi, 2");                                          // required headroom: four tables
    abi::emit_load_symbol_to_reg(emitter, "r8", "_heap_max", 0);
    abi::emit_load_symbol_to_reg(emitter, "r9", "_heap_off", 0);
    emitter.instruction("cmp r9, r8");                                          // is the bump offset already at or past the arena end?
    emitter.instruction("jae __rt_json_match_build_done_x");                    // no unbumped tail: keep the plain scan
    emitter.instruction("sub r8, r9");                                          // unbumped arena bytes
    emitter.instruction("cmp r8, rsi");                                         // enough room for the table and the decode?
    emitter.instruction("jb __rt_json_match_build_done_x");                     // tight arena: keep the plain scan
    abi::emit_load_symbol_to_reg(emitter, "r8", "_heap_limit", 0);
    abi::emit_load_symbol_to_reg(emitter, "r9", "_gc_live", 0);
    emitter.instruction("cmp r9, r8");                                          // is the live footprint already at the ceiling?
    emitter.instruction("jae __rt_json_match_build_done_x");                    // no budget left: keep the plain scan
    emitter.instruction("sub r8, r9");                                          // remaining live-memory budget
    emitter.instruction("cmp r8, rsi");                                         // enough budget for the table and the decode?
    emitter.instruction("jb __rt_json_match_build_done_x");                     // tight budget: keep the plain scan

    // -- allocate the header plus one u32 match slot per document byte --
    emitter.instruction("lea rax, [rdx * 4 + 16]");                             // table size in bytes
    emitter.instruction("call __rt_heap_alloc");                                // rax = table payload
    emitter.instruction("mov rdi, rax");                                        // keep the table in a register the pass does not reuse
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the document pointer
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // reload the document length
    emitter.instruction("mov QWORD PTR [rdi], rax");                            // record the document base the offsets refer to
    emitter.instruction("mov QWORD PTR [rdi + 8], rdx");                        // record the document length

    // -- pass 2: pair brackets; open slots chain the stack until their close arrives --
    emitter.instruction("xor ecx, ecx");                                        // byte index
    emitter.instruction("mov r8d, -1");                                         // empty open-bracket stack
    emitter.instruction("xor r10d, r10d");                                      // inside-string flag
    emitter.instruction("xor r11d, r11d");                                      // pending-escape flag
    emitter.label("__rt_json_match_pair_loop_x");
    emitter.instruction("cmp rcx, rdx");                                        // every document byte visited?
    emitter.instruction("jae __rt_json_match_pair_done_x");                     // pairing is complete
    emitter.instruction("movzx esi, BYTE PTR [rax + rcx]");                     // load the current byte
    emitter.instruction("test r11d, r11d");                                     // is this byte escaped?
    emitter.instruction("jne __rt_json_match_pair_unescape_x");                 // an escaped byte is consumed verbatim
    emitter.instruction("test r10d, r10d");                                     // inside a string?
    emitter.instruction("jne __rt_json_match_pair_string_x");                   // only quotes and escapes matter there
    emitter.instruction("cmp esi, 34");                                         // '"'
    emitter.instruction("je __rt_json_match_pair_enter_string_x");              // a string starts
    emitter.instruction("cmp esi, 91");                                         // '['
    emitter.instruction("je __rt_json_match_pair_open_x");                      // an array opens
    emitter.instruction("cmp esi, 123");                                        // '{'
    emitter.instruction("je __rt_json_match_pair_open_x");                      // an object opens
    emitter.instruction("cmp esi, 93");                                         // ']'
    emitter.instruction("je __rt_json_match_pair_close_x");                     // a container closes
    emitter.instruction("cmp esi, 125");                                        // '}'
    emitter.instruction("je __rt_json_match_pair_close_x");                     // a container closes
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // any other byte is not a bracket
    emitter.label("__rt_json_match_pair_open_x");
    emitter.instruction("mov DWORD PTR [rdi + rcx * 4 + 16], r8d");             // link this open to the enclosing one
    emitter.instruction("mov r8d, ecx");                                        // this open is the new stack top
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_close_x");
    emitter.instruction("cmp r8d, -1");                                         // a stray close with no open?
    emitter.instruction("je __rt_json_match_pair_next_x");                      // stray closes pair with nothing
    emitter.instruction("mov r9d, r8d");                                        // the innermost open closes here
    emitter.instruction("mov r8d, DWORD PTR [rdi + r9 * 4 + 16]");              // pop back to the enclosing open
    emitter.instruction("mov DWORD PTR [rdi + r9 * 4 + 16], ecx");              // record the matching close offset
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_enter_string_x");
    emitter.instruction("mov r10d, 1");                                         // the scan is now inside a string
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_string_x");
    emitter.instruction("cmp esi, 92");                                         // '\\'
    emitter.instruction("je __rt_json_match_pair_escape_x");                    // the next byte is escaped
    emitter.instruction("cmp esi, 34");                                         // '"'
    emitter.instruction("jne __rt_json_match_pair_next_x");                     // ordinary string byte
    emitter.instruction("xor r10d, r10d");                                      // the string ends
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_escape_x");
    emitter.instruction("mov r11d, 1");                                         // remember the pending escape
    emitter.instruction("jmp __rt_json_match_pair_next_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_unescape_x");
    emitter.instruction("xor r11d, r11d");                                      // the escaped byte is consumed
    emitter.label("__rt_json_match_pair_next_x");
    emitter.instruction("add rcx, 1");                                          // advance to the next byte
    emitter.instruction("jmp __rt_json_match_pair_loop_x");                     // continue pairing
    emitter.label("__rt_json_match_pair_done_x");
    emitter.instruction("cmp r8d, -1");                                         // any open left without a close?
    emitter.instruction("je __rt_json_match_publish_x");                        // every open was paired
    emitter.instruction("mov r9d, r8d");                                        // take the innermost unmatched open
    emitter.instruction("mov r8d, DWORD PTR [rdi + r9 * 4 + 16]");              // pop back to the enclosing open
    emitter.instruction("mov DWORD PTR [rdi + r9 * 4 + 16], -1");               // mark the open as unmatched
    emitter.instruction("jmp __rt_json_match_pair_done_x");                     // drain the rest of the stack
    emitter.label("__rt_json_match_publish_x");
    abi::emit_store_reg_to_symbol(emitter, "rdi", "_json_match_table", 0);

    emitter.label("__rt_json_match_build_done_x");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // restore the document pointer for the decoder
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // restore the document length for the decoder
    emitter.instruction("mov rsp, rbp");                                        // release the pre-pass frame
    emitter.instruction("pop rbp");                                             // restore the boundary frame pointer
    emitter.instruction("ret");                                                 // return to the decoder boundary
}

/// Emits the scanner fast path for an opening bracket at `[slice_ptr + cursor]`.
///
/// Expects `slice_ptr` in rax, the cursor in rcx and the scan limit in rdx; clobbers r8 and r9.
/// When the table records a close strictly inside the scan limit, jumps to `skip_label` with
/// the cursor on that close (the scanner's advance step then steps past it at unchanged depth);
/// otherwise falls through so the caller performs its ordinary depth increment.
pub(super) fn emit_scanner_skip_x86_64(emitter: &mut Emitter, skip_label: &str, prefix: &str) {
    let fallthrough = format!("{prefix}_match_miss_x");
    abi::emit_load_symbol_to_reg(emitter, "r9", "_json_match_table", 0);
    emitter.instruction("test r9, r9");                                         // was a match table built for this document?
    emitter.instruction(&format!("je {fallthrough}"));                          // no table: keep the plain depth walk
    emitter.instruction("lea r8, [rax + rcx]");                                 // absolute address of the opening bracket
    emitter.instruction("sub r8, QWORD PTR [r9]");                              // offset of the bracket inside the table's document
    emitter.instruction("cmp r8, QWORD PTR [r9 + 8]");                          // does the bracket belong to that document?
    emitter.instruction(&format!("jae {fallthrough}"));                         // foreign or out-of-range slice: plain walk
    emitter.instruction("mov r8d, DWORD PTR [r9 + r8 * 4 + 16]");               // matching close offset (zero-extended)
    emitter.instruction("cmp r8d, -1");                                         // was this open left unmatched?
    emitter.instruction(&format!("je {fallthrough}"));                          // unmatched opens keep the plain walk
    emitter.instruction("add r8, QWORD PTR [r9]");                              // absolute address of the matching close
    emitter.instruction("sub r8, rax");                                         // close position relative to this slice
    emitter.instruction("cmp r8, rcx");                                         // does the close lie after the opening bracket?
    emitter.instruction(&format!("jle {fallthrough}"));                         // never move the cursor backwards
    emitter.instruction("cmp r8, rdx");                                         // does the close lie inside the scanned range?
    emitter.instruction(&format!("jge {fallthrough}"));                         // only a close the plain walk would reach is used
    emitter.instruction("mov rcx, r8");                                         // jump the cursor onto the matching close
    emitter.instruction(&format!("jmp {skip_label}"));                          // step past it with the depth unchanged
    emitter.label(&fallthrough);
}

/// Emits the boundary epilogue that releases a published match table. Preserves rax.
pub(super) fn emit_release_x86_64(emitter: &mut Emitter) {
    emitter.instruction("push rax");                                            // preserve the decoder result
    emitter.instruction("sub rsp, 8");                                          // keep the release call aligned
    abi::emit_load_symbol_to_reg(emitter, "rax", "_json_match_table", 0);
    emitter.instruction("test rax, rax");                                       // was a table published?
    emitter.instruction("je __rt_json_match_release_skip_x");                   // nothing to release
    abi::emit_store_zero_to_symbol(emitter, "_json_match_table", 0);
    emitter.instruction("call __rt_heap_free");                                 // release the table storage
    emitter.label("__rt_json_match_release_skip_x");
    emitter.instruction("add rsp, 8");                                          // drop the alignment pad
    emitter.instruction("pop rax");                                             // restore the decoder result
}
