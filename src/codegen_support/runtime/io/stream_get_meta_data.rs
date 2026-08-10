//! Purpose:
//! Emits the `__rt_stream_get_meta_data` runtime helper, which builds the
//! PHP-compatible metadata hash describing an open stream resource.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - Returns a `{string => mixed}` hash with the nine documented keys. `eof`
//!   comes from the stable `StreamState`; `seekable`/`stream_type` are derived
//!   from `lseek`; `blocked`/`mode` from `fcntl(F_GETFL)`.
//! - `wrapper_type` and `uri` come from handle-keyed StreamState metadata.

use crate::codegen_support::abi;
use crate::codegen_support::runtime::resources::layout::{
    STREAM_FD_OFFSET, STREAM_MODE_LEN_OFFSET, STREAM_MODE_PTR_OFFSET, STREAM_URI_LEN_OFFSET,
    STREAM_URI_PTR_OFFSET, STREAM_WRAPPER_ID_OFFSET,
};
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// stream_get_meta_data: build the metadata hash for an opaque stream handle.
/// Input:  AArch64 x0 = handle / x86_64 rdi = handle
/// Output: pointer to a `{string => mixed}` hash table
pub fn emit_stream_get_meta_data(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_stream_get_meta_data_linux_x86_64(emitter);
        return;
    }

    let plat = emitter.platform;
    let nonblock = plat.o_nonblock();

    emitter.blank();
    emitter.comment("--- runtime: stream_get_meta_data ---");
    emitter.label_global("__rt_stream_get_meta_data");

    // Frame (112 bytes): [0]=handle [8]=hash [16]=seekable [24]=blocked [32]=eof
    //                   [40]=mode_ptr [48]=mode_len [56]=stype_ptr [64]=stype_len
    //                   [72]=backend fd [80]=StreamState [96]=x29 [104]=x30
    emitter.instruction("sub sp, sp, #112");                                    // allocate the metadata frame
    emitter.instruction("stp x29, x30, [sp, #96]");                             // save frame pointer and return address
    emitter.instruction("add x29, sp, #96");                                    // establish the helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the opaque stream handle
    emitter.instruction("bl __rt_stream_state");                                // resolve handle-keyed metadata and backend state
    emitter.instruction("str x0, [sp, #80]");                                   // preserve StreamState across metadata hash construction
    emitter.instruction(&format!(
        "ldr x0, [x0, #{}]", STREAM_FD_OFFSET
    ));                                                                         // load the backend descriptor for native probes
    emitter.instruction("str x0, [sp, #72]");                                   // preserve the descriptor for native metadata probes

    // -- seekability: lseek(fd, 0, SEEK_CUR) --
    emitter.instruction("mov x1, #0");                                          // offset 0
    emitter.instruction("mov x2, #1");                                          // SEEK_CUR
    emitter.syscall(199);
    if plat.needs_cmp_before_error_branch() {
        emitter.instruction("cmp x0, #0");                                      // Linux: a negative result means lseek failed
    }
    emitter.instruction(&plat.branch_on_syscall_success("__rt_sgmd_seekable")); // lseek ok: the stream is seekable

    // -- not seekable: socket-like stream --
    emitter.instruction("mov x9, #0");                                          // seekable = false
    emitter.instruction("str x9, [sp, #16]");                                   // save the seekable flag
    abi::emit_symbol_address(emitter, "x10", "_meta_stype_socket");             // load page of the "tcp_socket" literal
    emitter.instruction("str x10, [sp, #56]");                                  // save the stream_type pointer
    emitter.instruction("mov x10, #10");                                        // length of "tcp_socket"
    emitter.instruction("str x10, [sp, #64]");                                  // save the stream_type length
    emitter.instruction("b __rt_sgmd_seek_done");                               // skip the seekable branch

    emitter.label("__rt_sgmd_seekable");
    emitter.instruction("mov x9, #1");                                          // seekable = true
    emitter.instruction("str x9, [sp, #16]");                                   // save the seekable flag
    abi::emit_symbol_address(emitter, "x10", "_meta_stype_stdio");              // load page of the "STDIO" literal
    emitter.instruction("str x10, [sp, #56]");                                  // save the stream_type pointer
    emitter.instruction("mov x10, #5");                                         // length of "STDIO"
    emitter.instruction("str x10, [sp, #64]");                                  // save the stream_type length
    emitter.label("__rt_sgmd_seek_done");
    // `stream_type` is a wrapper and backend identity in php-src, not a descriptor property. The
    // derivation above only knows whether `lseek` worked, which called `php://memory` STDIO and a
    // `popen()` pipe a socket; a stream that records an identity reports that instead.
    emitter.instruction("ldr x0, [sp, #0]");                                    // the opaque stream handle
    emitter.instruction("bl __rt_stream_type_name");                            // x0 = name or 0, x1 = length
    emitter.instruction("cbz x0, __rt_sgmd_stype_kept");                        // nothing recorded: keep the derived name
    emitter.instruction("str x0, [sp, #56]");                                   // report the recorded name
    emitter.instruction("str x1, [sp, #64]");                                   // and its length
    emitter.label("__rt_sgmd_stype_kept");

    // -- blocking mode + access mode: fcntl(fd, F_GETFL, 0) --
    emitter.instruction("ldr x0, [sp, #72]");                                   // reload the stream descriptor
    emitter.instruction("mov x1, #3");                                          // F_GETFL
    emitter.instruction("mov x2, #0");                                          // unused third argument
    emitter.syscall(92);
    emitter.instruction(&format!("mov x9, #{}", nonblock));                     // the O_NONBLOCK flag bit
    emitter.instruction("tst x0, x9");                                          // is the O_NONBLOCK bit set?
    emitter.instruction("cset x10, eq");                                        // blocked = 1 when O_NONBLOCK is clear
    emitter.instruction("str x10, [sp, #24]");                                  // save the blocked flag
    emitter.instruction("and x9, x0, #3");                                      // isolate the O_ACCMODE access bits
    emitter.instruction("cmp x9, #1");                                          // O_WRONLY?
    emitter.instruction("b.eq __rt_sgmd_mode_w");                               // write-only stream
    emitter.instruction("cmp x9, #2");                                          // O_RDWR?
    emitter.instruction("b.eq __rt_sgmd_mode_rw");                              // read-write stream

    abi::emit_symbol_address(emitter, "x10", "_meta_mode_r");                   // load page of the "r" literal
    emitter.instruction("mov x11, #1");                                         // length of "r"
    emitter.instruction("b __rt_sgmd_mode_done");                               // mode resolved
    emitter.label("__rt_sgmd_mode_w");
    abi::emit_symbol_address(emitter, "x10", "_meta_mode_w");                   // load page of the "w" literal
    emitter.instruction("mov x11, #1");                                         // length of "w"
    emitter.instruction("b __rt_sgmd_mode_done");                               // mode resolved
    emitter.label("__rt_sgmd_mode_rw");
    abi::emit_symbol_address(emitter, "x10", "_meta_mode_rw");                  // load page of the "r+" literal
    emitter.instruction("mov x11, #2");                                         // length of "r+"
    emitter.label("__rt_sgmd_mode_done");
    // A mode recorded at open time is what PHP reports; the derivation above is only the
    // fallback for streams that never recorded one, and it cannot spell `a`, `w+` or `rb`.
    emitter.instruction("ldr x12, [sp, #0]");                                   // the opaque stream handle
    emitter.instruction("str x10, [sp, #40]");                                  // save the derived mode pointer
    emitter.instruction("str x11, [sp, #48]");                                  // save the derived mode length
    emitter.instruction("mov x0, x12");
    emitter.instruction("bl __rt_stream_state");                                // resolve the owning stream state
    emitter.instruction("cbz x0, __rt_sgmd_mode_kept");                         // no state: keep the derived spelling
    emitter.instruction(&format!("ldr x9, [x0, #{STREAM_MODE_PTR_OFFSET}]"));   // the recorded mode
    emitter.instruction("cbz x9, __rt_sgmd_mode_kept");                         // nothing recorded: keep the derived spelling
    emitter.instruction("str x9, [sp, #40]");                                   // report the recorded pointer
    emitter.instruction(&format!("ldr x9, [x0, #{STREAM_MODE_LEN_OFFSET}]"));
    emitter.instruction("str x9, [sp, #48]");                                   // and its length
    emitter.label("__rt_sgmd_mode_kept");

    // -- end-of-file flag from the authoritative StreamState --
    emitter.instruction("ldr x0, [sp, #0]");                                    // reload the opaque stream handle
    emitter.instruction("bl __rt_stream_eof_get");                              // read the state-owned EOF predicate
    emitter.instruction("str x0, [sp, #32]");                                   // save the EOF flag

    // -- create the metadata hash (capacity 16, value type = mixed) --
    emitter.instruction("mov x0, #16");                                         // initial capacity
    emitter.instruction("mov x1, #7");                                          // value type = mixed
    emitter.instruction("bl __rt_hash_new");                                    // allocate the hash; x0 = hash pointer
    emitter.instruction("str x0, [sp, #8]");                                    // save the hash pointer

    emit_set_bool_const(emitter, "_meta_key_timed_out", 9, 0);
    emit_set_bool_slot(emitter, "_meta_key_blocked", 7, 24);
    emit_set_bool_slot(emitter, "_meta_key_eof", 3, 32);
    emit_set_int_const(emitter, "_meta_key_unread_bytes", 12);
    emit_set_str_slots(emitter, "_meta_key_stream_type", 11, 56, 64);
    // -- wrapper_type: map the StreamState wrapper id to its PHP-visible literal --
    emit_set_wrapper_type_aarch64(emitter);
    emit_set_owned_str_slots(emitter, "_meta_key_mode", 4, 40, 48);
    emit_set_bool_slot(emitter, "_meta_key_seekable", 8, 16);
    // -- uri: read the StreamState-owned URI pointer/length pair --
    emit_set_uri_aarch64(emitter);

    // -- return the completed hash --
    emitter.instruction("ldr x0, [sp, #8]");                                    // load the final hash pointer
    emitter.instruction("ldp x29, x30, [sp, #96]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #112");                                    // release the metadata frame
    emitter.instruction("ret");                                                 // return the metadata hash pointer
}

/// Reads the StreamState wrapper id and loads the matching wrapper-type string
/// literal into x3 (ptr) / x4 (len) / x5 (tag=1), then inserts the hash entry.
/// Fallback id 0 (unset) → "plainfile".
fn emit_set_wrapper_type_aarch64(emitter: &mut Emitter) {
    let wrappers: &[(&str, i64)] = &[
        ("_meta_wrapper_plainfile", 9),
        ("_meta_wrapper_http", 4),
        ("_meta_wrapper_https", 5),
        ("_meta_wrapper_ftp", 3),
        ("_meta_wrapper_ftps", 4),
        ("_meta_wrapper_phar", 4),
        ("_meta_wrapper_php", 3),
        ("_meta_wrapper_data", 7),
        ("_meta_wrapper_zlib", 13),
        ("_meta_wrapper_bzip2", 14),
        ("_meta_wrapper_glob", 4),
        ("_meta_wrapper_user", 10),
    ];
    emitter.instruction("ldr x6, [sp, #80]");                                   // reload the stable StreamState pointer
    emitter.instruction(&format!(
        "ldr x7, [x6, #{}]", STREAM_WRAPPER_ID_OFFSET
    ));                                                                         // load the handle-keyed wrapper id
    // Compare-and-branch chain: each comparison branches to a label that is
    // emitted after all comparisons, so the fall-through goes to the next
    // comparison rather than into the literal-load block.
    for (id, _) in wrappers.iter().enumerate() {
        let label = format!("__rt_sgmd_wid_{}", id);
        emitter.instruction(&format!("cmp w7, #{}", id));                       // compare wrapper id
        emitter.instruction(&format!("b.eq {}", label));                        // branch to the matching literal load
    }
    // Fallback for unknown ids: use plainfile (same as id 0).
    abi::emit_symbol_address(emitter, "x3", "_meta_wrapper_plainfile");          // fallback wrapper name
    emitter.instruction("mov x4, #9");                                          // plainfile length
    emitter.instruction("mov x5, #1");                                          // value tag = string
    emitter.instruction("b __rt_sgmd_wtype_put");                               // jump to the shared hash insert
    // Literal-load blocks (emitted after the compare chain).
    for (id, (sym, len)) in wrappers.iter().enumerate() {
        let label = format!("__rt_sgmd_wid_{}", id);
        emitter.label(&label);
        abi::emit_symbol_address(emitter, "x3", sym);                            // load the wrapper-name literal address
        emitter.instruction(&format!("mov x4, #{}", len));                      // wrapper-name length
        emitter.instruction("mov x5, #1");                                      // value tag = string
        emitter.instruction("b __rt_sgmd_wtype_put");                           // jump to the shared hash insert
    }
    emitter.label("__rt_sgmd_wtype_put");
    emit_hash_put_aarch64(emitter, "_meta_key_wrapper_type", 12);
}

/// Reads the StreamState URI pointer/length pair and loads it into
/// x3 (ptr) / x4 (len) / x5 (tag=1), then inserts the hash entry.
/// Fallback (ptr == 0) → empty string.
fn emit_set_uri_aarch64(emitter: &mut Emitter) {
    emitter.instruction("ldr x6, [sp, #80]");                                   // reload the stable StreamState pointer
    emitter.instruction(&format!(
        "ldr x3, [x6, #{}]", STREAM_URI_PTR_OFFSET
    ));                                                                         // load the handle-keyed URI pointer
    emitter.instruction(&format!(
        "ldr x4, [x6, #{}]", STREAM_URI_LEN_OFFSET
    ));                                                                         // load the handle-keyed URI byte length
    emitter.instruction("cbz x3, __rt_sgmd_uri_empty");                         // null ptr → empty uri
    // The array releases its string values, so handing it the StreamState's own URI allocation
    // freed the state's copy: the third `stream_get_meta_data()` on a stream then read a block
    // that intervening hash keys had already reused. Give the array a duplicate it can own.
    emitter.instruction("mov x1, x3");                                          // duplicate the URI bytes
    emitter.instruction("mov x2, x4");                                          // with their length
    emitter.instruction("bl __rt_str_persist");                                 // into storage the array may release
    emitter.instruction("mov x3, x1");                                          // value_lo = the owned duplicate
    emitter.instruction("mov x4, x2");                                          // value_hi = its length
    emitter.instruction("mov x5, #1");                                          // value tag = string
    emitter.instruction("b __rt_sgmd_uri_put");                                 // insert the uri entry
    emitter.label("__rt_sgmd_uri_empty");
    emitter.instruction("mov x3, #0");                                          // ptr = null (empty string)
    emitter.instruction("mov x4, #0");                                          // len = 0
    emitter.instruction("mov x5, #1");                                          // value tag = string
    emitter.label("__rt_sgmd_uri_put");
    emit_hash_put_aarch64(emitter, "_meta_key_uri", 3);
}

/// Emit one `__rt_hash_set` with the value already staged in x3/x4/x5.
fn emit_hash_put_aarch64(emitter: &mut Emitter, key_sym: &str, key_len: i64) {
    emitter.instruction("ldr x0, [sp, #8]");                                    // reload the hash pointer
    abi::emit_symbol_address(emitter, "x1", key_sym);                           // load page of the key literal
    emitter.instruction(&format!("mov x2, #{}", key_len));                      // key length
    emitter.instruction("bl __rt_hash_set");                                    // insert the entry; x0 = updated hash
    emitter.instruction("str x0, [sp, #8]");                                    // persist any post-grow hash pointer
}

/// Emits the set bool const stream runtime helper.
fn emit_set_bool_const(emitter: &mut Emitter, key_sym: &str, key_len: i64, value: i64) {
    emitter.instruction(&format!("mov x3, #{}", value));                        // value_lo = boolean payload
    emitter.instruction("mov x4, #0");                                          // value_hi unused for booleans
    emitter.instruction("mov x5, #3");                                          // value tag = bool
    emit_hash_put_aarch64(emitter, key_sym, key_len);
}

/// Emits the set bool slot stream runtime helper.
fn emit_set_bool_slot(emitter: &mut Emitter, key_sym: &str, key_len: i64, slot: i64) {
    emitter.instruction(&format!("ldr x3, [sp, #{}]", slot));                   // value_lo = computed boolean
    emitter.instruction("mov x4, #0");                                          // value_hi unused for booleans
    emitter.instruction("mov x5, #3");                                          // value tag = bool
    emit_hash_put_aarch64(emitter, key_sym, key_len);
}

/// Emits the set int const stream runtime helper.
fn emit_set_int_const(emitter: &mut Emitter, key_sym: &str, key_len: i64) {
    emitter.instruction("mov x3, #0");                                          // value_lo = 0 (elephc keeps no read buffer)
    emitter.instruction("mov x4, #0");                                          // value_hi unused for integers
    emitter.instruction("mov x5, #0");                                          // value tag = int
    emit_hash_put_aarch64(emitter, key_sym, key_len);
}

/// Emits the set str slots stream runtime helper.
fn emit_set_str_slots(emitter: &mut Emitter, key_sym: &str, key_len: i64, ptr_slot: i64, len_slot: i64) {
    emitter.instruction(&format!("ldr x3, [sp, #{}]", ptr_slot));               // value_lo = string pointer
    emitter.instruction(&format!("ldr x4, [sp, #{}]", len_slot));               // value_hi = string length
    emitter.instruction("mov x5, #1");                                          // value tag = string
    emit_hash_put_aarch64(emitter, key_sym, key_len);
}

/// Emits one `__rt_hash_set` whose string value is duplicated first.
///
/// See the URI insertion: a value the array may release must not be the StreamState's own
/// allocation. A rodata fallback survives either way, so this is uniform for both.
fn emit_set_owned_str_slots(
    emitter: &mut Emitter,
    key_sym: &str,
    key_len: i64,
    ptr_slot: i64,
    len_slot: i64,
) {
    emitter.instruction(&format!("ldr x1, [sp, #{}]", ptr_slot));               // duplicate the recorded bytes
    emitter.instruction(&format!("ldr x2, [sp, #{}]", len_slot));               // with their length
    emitter.instruction("bl __rt_str_persist");                                 // into storage the array may release
    emitter.instruction("mov x3, x1");                                          // value_lo = the owned duplicate
    emitter.instruction("mov x4, x2");                                          // value_hi = its length
    emitter.instruction("mov x5, #1");                                          // value tag = string
    emit_hash_put_aarch64(emitter, key_sym, key_len);
}

/// Emits the Linux x86_64 stream runtime helper for stream get meta data.
fn emit_stream_get_meta_data_linux_x86_64(emitter: &mut Emitter) {
    let plat = emitter.platform;
    let nonblock = plat.o_nonblock();

    emitter.blank();
    emitter.comment("--- runtime: stream_get_meta_data ---");
    emitter.label_global("__rt_stream_get_meta_data");

    // Frame (rbp-relative): [-8]=handle [-16]=hash [-24]=seekable [-32]=blocked
    //                       [-40]=eof [-48]=mode_ptr [-56]=mode_len
    //                       [-64]=stype_ptr [-72]=stype_len [-80]=backend fd
    //                       [-88]=StreamState
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish the helper frame pointer
    emitter.instruction("sub rsp, 96");                                         // reserve aligned metadata spill slots
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the opaque stream handle
    emitter.instruction("call __rt_stream_state");                              // resolve handle-keyed metadata and backend state
    emitter.instruction("mov QWORD PTR [rbp - 88], rax");                       // preserve StreamState across metadata hash construction
    emitter.instruction(&format!(
        "mov rdi, QWORD PTR [rax + {}]", STREAM_FD_OFFSET
    ));                                                                         // load the backend descriptor for native probes
    emitter.instruction("mov QWORD PTR [rbp - 80], rdi");                       // preserve the descriptor for native metadata probes

    // -- seekability: lseek(fd, 0, SEEK_CUR) --
    emitter.instruction("xor esi, esi");                                        // offset 0
    emitter.instruction("mov edx, 1");                                          // SEEK_CUR
    emitter.instruction("mov eax, 8");                                          // Linux x86_64 syscall 8 = lseek
    emitter.instruction("syscall");                                             // probe whether the descriptor is seekable
    emitter.instruction("test rax, rax");                                       // did lseek fail with a negative result?
    emitter.instruction("jns __rt_sgmd_seekable_x86");                          // lseek ok: the stream is seekable

    emitter.instruction("mov QWORD PTR [rbp - 24], 0");                         // seekable = false
    abi::emit_symbol_address(emitter, "r10", "_meta_stype_socket");             // address of the "tcp_socket" literal
    emitter.instruction("mov QWORD PTR [rbp - 64], r10");                       // save the stream_type pointer
    emitter.instruction("mov QWORD PTR [rbp - 72], 10");                        // save the stream_type length
    emitter.instruction("jmp __rt_sgmd_seek_done_x86");                         // skip the seekable branch

    emitter.label("__rt_sgmd_seekable_x86");
    emitter.instruction("mov QWORD PTR [rbp - 24], 1");                         // seekable = true
    abi::emit_symbol_address(emitter, "r10", "_meta_stype_stdio");              // address of the "STDIO" literal
    emitter.instruction("mov QWORD PTR [rbp - 64], r10");                       // save the stream_type pointer
    emitter.instruction("mov QWORD PTR [rbp - 72], 5");                         // save the stream_type length
    emitter.label("__rt_sgmd_seek_done_x86");
    // See the AArch64 counterpart: a recorded identity outranks the seekability-derived name.
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the opaque stream handle
    emitter.instruction("call __rt_stream_type_name");                          // rax = name or 0, rdx = length
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_sgmd_stype_kept_x86");                         // nothing recorded: keep the derived name
    emitter.instruction("mov QWORD PTR [rbp - 64], rax");                       // report the recorded name
    emitter.instruction("mov QWORD PTR [rbp - 72], rdx");                       // and its length
    emitter.label("__rt_sgmd_stype_kept_x86");

    // -- blocking mode + access mode: fcntl(fd, F_GETFL, 0) --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 80]");                       // reload the stream descriptor
    emitter.instruction("mov esi, 3");                                          // F_GETFL
    emitter.instruction("xor edx, edx");                                        // unused third argument
    emitter.instruction("mov eax, 72");                                         // Linux x86_64 syscall 72 = fcntl
    emitter.instruction("syscall");                                             // read the descriptor flags
    emitter.instruction(&format!("mov r9d, {}", nonblock));                     // the O_NONBLOCK flag bit
    emitter.instruction("test rax, r9");                                        // is the O_NONBLOCK bit set?
    emitter.instruction("sete r10b");                                           // blocked = 1 when O_NONBLOCK is clear
    emitter.instruction("movzx r10, r10b");                                     // widen the blocked flag to a full word
    emitter.instruction("mov QWORD PTR [rbp - 32], r10");                       // save the blocked flag
    emitter.instruction("and rax, 3");                                          // isolate the O_ACCMODE access bits
    emitter.instruction("cmp rax, 1");                                          // O_WRONLY?
    emitter.instruction("je __rt_sgmd_mode_w_x86");                             // write-only stream
    emitter.instruction("cmp rax, 2");                                          // O_RDWR?
    emitter.instruction("je __rt_sgmd_mode_rw_x86");                            // read-write stream

    abi::emit_symbol_address(emitter, "r10", "_meta_mode_r");                   // address of the "r" literal
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save the mode pointer
    emitter.instruction("mov QWORD PTR [rbp - 56], 1");                         // save the mode length
    emitter.instruction("jmp __rt_sgmd_mode_done_x86");                         // mode resolved
    emitter.label("__rt_sgmd_mode_w_x86");
    abi::emit_symbol_address(emitter, "r10", "_meta_mode_w");                   // address of the "w" literal
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save the mode pointer
    emitter.instruction("mov QWORD PTR [rbp - 56], 1");                         // save the mode length
    emitter.instruction("jmp __rt_sgmd_mode_done_x86");                         // mode resolved
    emitter.label("__rt_sgmd_mode_rw_x86");
    abi::emit_symbol_address(emitter, "r10", "_meta_mode_rw");                  // address of the "r+" literal
    emitter.instruction("mov QWORD PTR [rbp - 48], r10");                       // save the mode pointer
    emitter.instruction("mov QWORD PTR [rbp - 56], 2");                         // save the mode length
    emitter.label("__rt_sgmd_mode_done_x86");
    // See the AArch64 counterpart: a mode recorded at open time is what PHP reports, and the
    // derivation above is only the fallback for streams that never recorded one.
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // the opaque stream handle
    emitter.instruction("call __rt_stream_state");                              // resolve the owning stream state
    emitter.instruction("test rax, rax");
    emitter.instruction("jz __rt_sgmd_mode_kept_x86");                          // no state: keep the derived spelling
    emitter.instruction(&format!(
        "mov r9, QWORD PTR [rax + {STREAM_MODE_PTR_OFFSET}]"
    ));                                                                         // the recorded mode
    emitter.instruction("test r9, r9");
    emitter.instruction("jz __rt_sgmd_mode_kept_x86");                          // nothing recorded: keep the derived spelling
    emitter.instruction("mov QWORD PTR [rbp - 48], r9");                        // report the recorded pointer
    emitter.instruction(&format!(
        "mov r9, QWORD PTR [rax + {STREAM_MODE_LEN_OFFSET}]"
    ));
    emitter.instruction("mov QWORD PTR [rbp - 56], r9");                        // and its length
    emitter.label("__rt_sgmd_mode_kept_x86");

    // -- end-of-file flag from the authoritative StreamState --
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // reload the opaque stream handle
    emitter.instruction("call __rt_stream_eof_get");                            // read the state-owned EOF predicate
    emitter.instruction("mov QWORD PTR [rbp - 40], rax");                       // save the EOF flag

    // -- create the metadata hash (capacity 16, value type = mixed) --
    emitter.instruction("mov rdi, 16");                                         // initial capacity
    emitter.instruction("mov rsi, 7");                                          // value type = mixed
    emitter.instruction("call __rt_hash_new");                                  // allocate the hash; rax = hash pointer
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // save the hash pointer

    emit_set_bool_const_x86(emitter, "_meta_key_timed_out", 9, 0);
    emit_set_bool_slot_x86(emitter, "_meta_key_blocked", 7, 32);
    emit_set_bool_slot_x86(emitter, "_meta_key_eof", 3, 40);
    emit_set_int_const_x86(emitter, "_meta_key_unread_bytes", 12);
    emit_set_str_slots_x86(emitter, "_meta_key_stream_type", 11, 64, 72);
    // -- wrapper_type: map the StreamState wrapper id to its PHP-visible literal --
    emit_set_wrapper_type_x86(emitter);
    emit_set_owned_str_slots_x86(emitter, "_meta_key_mode", 4, 48, 56);
    emit_set_bool_slot_x86(emitter, "_meta_key_seekable", 8, 24);
    // -- uri: read the StreamState-owned URI pointer/length pair --
    emit_set_uri_x86(emitter);

    // -- return the completed hash --
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // load the final hash pointer
    emitter.instruction("add rsp, 96");                                         // release the metadata spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the metadata hash pointer
}

/// Emit one `__rt_hash_set` with the value already staged in rcx/r8/r9.
fn emit_hash_put_x86(emitter: &mut Emitter, key_sym: &str, key_len: i64) {
    abi::emit_symbol_address(emitter, "rsi", key_sym);                          // key pointer
    emitter.instruction(&format!("mov rdx, {}", key_len));                      // key length
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // hash pointer (first argument)
    emitter.instruction("call __rt_hash_set");                                  // insert the entry; rax = updated hash
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // persist any post-grow hash pointer
}

/// Emits the set bool const x86 stream runtime helper.
fn emit_set_bool_const_x86(emitter: &mut Emitter, key_sym: &str, key_len: i64, value: i64) {
    emitter.instruction(&format!("mov rcx, {}", value));                        // value_lo = boolean payload
    emitter.instruction("xor r8d, r8d");                                        // value_hi unused for booleans
    emitter.instruction("mov r9, 3");                                           // value tag = bool
    emit_hash_put_x86(emitter, key_sym, key_len);
}

/// Emits the set bool slot x86 stream runtime helper.
fn emit_set_bool_slot_x86(emitter: &mut Emitter, key_sym: &str, key_len: i64, slot: i64) {
    emitter.instruction(&format!("mov rcx, QWORD PTR [rbp - {}]", slot));       // value_lo = computed boolean
    emitter.instruction("xor r8d, r8d");                                        // value_hi unused for booleans
    emitter.instruction("mov r9, 3");                                           // value tag = bool
    emit_hash_put_x86(emitter, key_sym, key_len);
}

/// Emits the set int const x86 stream runtime helper.
fn emit_set_int_const_x86(emitter: &mut Emitter, key_sym: &str, key_len: i64) {
    emitter.instruction("xor ecx, ecx");                                        // value_lo = 0 (elephc keeps no read buffer)
    emitter.instruction("xor r8d, r8d");                                        // value_hi unused for integers
    emitter.instruction("xor r9d, r9d");                                        // value tag = int
    emit_hash_put_x86(emitter, key_sym, key_len);
}

/// Reads the StreamState wrapper id and loads the matching wrapper-type string
/// literal into rcx (ptr) / r8 (len) / r9 (tag=1), then inserts the hash entry.
/// Fallback id 0 (unset) → "plainfile".
fn emit_set_wrapper_type_x86(emitter: &mut Emitter) {
    let wrappers: &[(&str, i64)] = &[
        ("_meta_wrapper_plainfile", 9),
        ("_meta_wrapper_http", 4),
        ("_meta_wrapper_https", 5),
        ("_meta_wrapper_ftp", 3),
        ("_meta_wrapper_ftps", 4),
        ("_meta_wrapper_phar", 4),
        ("_meta_wrapper_php", 3),
        ("_meta_wrapper_data", 7),
        ("_meta_wrapper_zlib", 13),
        ("_meta_wrapper_bzip2", 14),
        ("_meta_wrapper_glob", 4),
        ("_meta_wrapper_user", 10),
    ];
    emitter.instruction("mov r10, QWORD PTR [rbp - 88]");                       // reload the stable StreamState pointer
    emitter.instruction(&format!(
        "mov rax, QWORD PTR [r10 + {}]", STREAM_WRAPPER_ID_OFFSET
    ));                                                                         // load the handle-keyed wrapper id
    for (id, _) in wrappers.iter().enumerate() {
        let label = format!("__rt_sgmd_wid_{}_x", id);
        emitter.instruction(&format!("cmp eax, {}", id));                       // compare wrapper id
        emitter.instruction(&format!("je {}", label));                          // branch to the matching literal load
    }
    // Fallback for unknown ids: use plainfile (same as id 0).
    abi::emit_symbol_address(emitter, "rcx", "_meta_wrapper_plainfile");          // fallback wrapper name
    emitter.instruction("mov r8, 9");                                           // plainfile length
    emitter.instruction("mov r9, 1");                                           // value tag = string
    emitter.instruction("jmp __rt_sgmd_wtype_put_x");                           // jump to the shared hash insert
    for (id, (sym, len)) in wrappers.iter().enumerate() {
        let label = format!("__rt_sgmd_wid_{}_x", id);
        emitter.label(&label);
        abi::emit_symbol_address(emitter, "rcx", sym);                            // load the wrapper-name literal address
        emitter.instruction(&format!("mov r8, {}", len));                       // wrapper-name length
        emitter.instruction("mov r9, 1");                                       // value tag = string
        emitter.instruction("jmp __rt_sgmd_wtype_put_x");                       // jump to the shared hash insert
    }
    emitter.label("__rt_sgmd_wtype_put_x");
    emit_hash_put_x86(emitter, "_meta_key_wrapper_type", 12);
}

/// Reads the StreamState URI pointer/length pair and loads it into
/// rcx (ptr) / r8 (len) / r9 (tag=1), then inserts the hash entry.
/// Fallback (ptr == 0) → empty string.
fn emit_set_uri_x86(emitter: &mut Emitter) {
    emitter.instruction("mov r10, QWORD PTR [rbp - 88]");                       // reload the stable StreamState pointer
    emitter.instruction(&format!(
        "mov rcx, QWORD PTR [r10 + {}]", STREAM_URI_PTR_OFFSET
    ));                                                                         // load the handle-keyed URI pointer
    emitter.instruction(&format!(
        "mov r8, QWORD PTR [r10 + {}]", STREAM_URI_LEN_OFFSET
    ));                                                                         // load the handle-keyed URI byte length
    emitter.instruction("test rcx, rcx");                                       // null ptr?
    emitter.instruction("jz __rt_sgmd_uri_empty_x");                            // → empty uri
    // See the AArch64 counterpart: the array releases its string values, so it needs a duplicate
    // rather than the StreamState's own URI allocation.
    emitter.instruction("mov rax, rcx");                                        // duplicate the URI bytes
    emitter.instruction("mov rdx, r8");                                         // with their length
    emitter.instruction("call __rt_str_persist");                               // into storage the array may release
    emitter.instruction("mov rcx, rax");                                        // value_lo = the owned duplicate
    emitter.instruction("mov r8, rdx");                                         // value_hi = its length
    emitter.instruction("mov r9, 1");                                           // value tag = string
    emitter.instruction("jmp __rt_sgmd_uri_put_x");                             // insert the uri entry
    emitter.label("__rt_sgmd_uri_empty_x");
    emitter.instruction("xor ecx, ecx");                                        // ptr = 0 (empty string)
    emitter.instruction("xor r8d, r8d");                                        // len = 0
    emitter.instruction("mov r9, 1");                                           // value tag = string
    emitter.label("__rt_sgmd_uri_put_x");
    emit_hash_put_x86(emitter, "_meta_key_uri", 3);
}

/// Emits one x86_64 `__rt_hash_set` whose string value is duplicated first.
fn emit_set_owned_str_slots_x86(
    emitter: &mut Emitter,
    key_sym: &str,
    key_len: i64,
    ptr_slot: i64,
    len_slot: i64,
) {
    emitter.instruction(&format!("mov rax, QWORD PTR [rbp - {}]", ptr_slot));   // duplicate the recorded bytes
    emitter.instruction(&format!("mov rdx, QWORD PTR [rbp - {}]", len_slot));   // with their length
    emitter.instruction("call __rt_str_persist");                               // into storage the array may release
    emitter.instruction("mov rcx, rax");                                        // value_lo = the owned duplicate
    emitter.instruction("mov r8, rdx");                                         // value_hi = its length
    emitter.instruction("mov r9, 1");                                           // value tag = string
    emit_hash_put_x86(emitter, key_sym, key_len);
}

/// Emits the set str slots x86 stream runtime helper.
fn emit_set_str_slots_x86(emitter: &mut Emitter, key_sym: &str, key_len: i64, ptr_slot: i64, len_slot: i64) {
    emitter.instruction(&format!("mov rcx, QWORD PTR [rbp - {}]", ptr_slot));   // value_lo = string pointer
    emitter.instruction(&format!("mov r8, QWORD PTR [rbp - {}]", len_slot));    // value_hi = string length
    emitter.instruction("mov r9, 1");                                           // value tag = string
    emit_hash_put_x86(emitter, key_sym, key_len);
}
