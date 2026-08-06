//! Purpose:
//! Emits the hand-authored WebAssembly (WAT) file-I/O runtime for the wasm32-wasi
//! backend: `fopen`, `fread`, `fwrite`, `fclose`, `file_get_contents`,
//! `file_put_contents`, `file_exists` and `unlink`.
//!
//! Called from:
//! - `crate::codegen_wasm::runtime::emit_command_runtime()`, after the WASI imports
//!   it shares and before the helpers that call `__rt_heap_alloc`/`__rt_str_persist`.
//!
//! Key details:
//! - WASI Preview 1 is CAPABILITY-based: a path can only be opened relative to a
//!   directory the host preopened. `__rt_wasi_dirfd` finds the first preopen by
//!   probing `fd_prestat_get` from fd 3 upward, and every path call uses it. With no
//!   preopen at all — `node` without `preopens`, `wasmer` without `--dir` — every
//!   open fails, which is the same `false` PHP answers for an unopenable path.
//! - A PHP stream handle is a boxed Mixed cell with the resource tag (9) carrying the
//!   WASI fd as its payload, so it flows through locals, arrays and arguments as any
//!   other boxed value does.
//! - Syscall scratch lives at `$__float_scratch + 0x3000`, in the 4 KiB the strtod
//!   bignums (0..0x1200) and the ftoa/itoa buffers (0x2000..0x3000) leave free.

use super::wat::WatModule;

/// Offset of this module's syscall scratch within the float-scratch region.
const IO_SCRATCH: u32 = 0x3000;

/// Adds the file-I/O runtime to `wm`. Requires the WASI path imports and the heap
/// and string runtimes, all of which the command runtime emits alongside it.
pub(super) fn emit_file_runtime(wm: &mut WatModule) {
    wm.add_raw_func(RT_FOPEN_FAILED);
    wm.add_raw_func(RT_STD_STREAM_FD);
    wm.add_raw_func(&rt_wasi_dirfd());
    wm.add_raw_func(&rt_fopen());
    wm.add_raw_func(&rt_stream_fd());
    wm.add_raw_func(&rt_fwrite());
    wm.add_raw_func(&rt_fread());
    wm.add_raw_func(&rt_fclose());
    wm.add_raw_func(&rt_file_exists());
    wm.add_raw_func(&rt_unlink());
    wm.add_raw_func(&rt_file_size());
    wm.add_raw_func(&rt_file_get_contents());
    wm.add_raw_func(&rt_file_put_contents());
}

/// `__rt_fopen_failed`: the boxed `false` every unopenable path answers, warning first.
///
/// The flag exists because `file_get_contents` opens through `__rt_fopen` and must emit its OWN
/// message rather than `fopen`'s — php-src and the native backend both name the function the
/// script actually called.
const RT_FOPEN_FAILED: &str = r#"(func $__rt_fopen_failed (param $warn i32) (result i32)
  (if (local.get $warn)
    (then (call $__rt_warn_fopen_failed)))
  (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))
"#;

/// `__rt_std_stream_fd`: the fd behind `php://stdout`, `php://stderr`, `php://stdin` or
/// `php://output`, and -1 for anything else.
///
/// These four are the part of the `php://` wrapper that is not a stream implementation at all —
/// they name the three fds the process already has, which is why they work here while
/// `php://memory` and `php://temp` do not. `php://output` is PHP's own alias for the output
/// buffer, which for a command with no buffering active is stdout.
///
/// The comparison is open-coded rather than done against a data segment so this stays
/// independent of the module's static-data layout, the same reason `__rt_echo_array_word`
/// builds its five bytes in scratch.
const RT_STD_STREAM_FD: &str = r#"(func $__rt_std_stream_fd (param $path i32) (param $len i64) (result i32)
  (local $c i32)
  (if (i32.and (i64.ne (local.get $len) (i64.const 11)) (i64.ne (local.get $len) (i64.const 12)))
    (then (return (i32.const -1))))                              ;; no php:// name is another length
  (if (i32.eqz (i32.and (i32.and
        (i32.and (i32.eq (i32.load8_u (local.get $path)) (i32.const 112))                       ;; 'p'
                 (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 1))) (i32.const 104))) ;; 'h'
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 2))) (i32.const 112))  ;; 'p'
                 (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 3))) (i32.const 58)))) ;; ':'
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 4))) (i32.const 47))   ;; '/'
                 (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 5))) (i32.const 47))))) ;; '/'
    (then (return (i32.const -1))))
  (local.set $c (i32.load8_u (i32.add (local.get $path) (i32.const 6))))
  (if (i32.eq (local.get $c) (i32.const 111))                    ;; "output": the buffer, i.e. stdout
    (then (return (select (i32.const 1) (i32.const -1)
      (i32.and (i64.eq (local.get $len) (i64.const 12))
               (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 7))) (i32.const 117)))))))
  (if (i32.ne (local.get $c) (i32.const 115))                    ;; every other name starts "std"
    (then (return (i32.const -1))))
  (if (i32.eqz (i32.and
        (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 7))) (i32.const 116))   ;; 't'
        (i32.eq (i32.load8_u (i32.add (local.get $path) (i32.const 8))) (i32.const 100)))) ;; 'd'
    (then (return (i32.const -1))))
  (local.set $c (i32.load8_u (i32.add (local.get $path) (i32.const 9))))
  (if (i64.eq (local.get $len) (i64.const 11))
    (then (return (select (i32.const 0) (i32.const -1)           ;; "stdin"
      (i32.eq (local.get $c) (i32.const 105))))))                ;; 'i'
  (if (i32.eq (local.get $c) (i32.const 111))                    ;; "stdout"
    (then (return (i32.const 1))))
  (if (i32.eq (local.get $c) (i32.const 101))                    ;; "stderr"
    (then (return (i32.const 2))))
  (i32.const -1))
"#;

/// `__rt_wasi_dirfd`: the fd of the first directory the host preopened, or -1.
///
/// WASI hands a module its filesystem authority as preopened fds starting at 3, and a
/// path is only meaningful relative to one of them. Probing `fd_prestat_get` upward
/// finds the first; a host that preopened nothing answers `EBADF` for every probe and
/// this answers -1, which every caller turns into PHP's `false`.
///
/// The scan stops at 32: a host that preopened more than that has directories this
/// cannot reach, which is a coverage limit rather than a wrong answer.
fn rt_wasi_dirfd() -> String {
    format!(
        r#"(func $__rt_wasi_dirfd (result i32)
  (local $fd i32)
  (local.set $fd (i32.const 3))                                  ;; preopens start at fd 3
  (block $done (loop $next
    (br_if $done (i32.ge_u (local.get $fd) (i32.const 32)))
    (if (i32.eqz (call $wasi_fd_prestat_get
          (local.get $fd)
          (i32.add (global.get $__float_scratch) (i32.const {prestat}))))
      (then (return (local.get $fd))))                           ;; first preopen wins
    (local.set $fd (i32.add (local.get $fd) (i32.const 1)))
    (br $next)))
  (i32.const -1))
"#,
        prestat = IO_SCRATCH + 0x20
    )
}

/// `__rt_fopen`: opens a path in a PHP mode and boxes the fd as a resource.
///
/// The mode's FIRST byte picks the open flags — `r` opens an existing file, `w`
/// creates and truncates, `a` creates and appends, `x` creates and fails if the file
/// exists, `c` creates without truncating — and a `+` anywhere after it adds the
/// other direction. The `b` and `t` suffixes name a line-ending translation this
/// target does not perform, so they are read and ignored, which is what they mean on
/// every platform PHP treats as binary-safe.
///
/// Rights are requested per direction rather than generously: `path_open` fails with
/// `ENOTCAPABLE` when a caller asks for more than the preopen holds, so asking for
/// what the mode actually needs is what makes a read-only preopen still serve `r`.
///
/// Any failure answers boxed `false`, which is what PHP answers for a path it cannot open,
/// after the warning the native backend emits there. `warn` is zero for the internal opens
/// `file_get_contents` and `file_put_contents` perform, which name themselves instead.
fn rt_fopen() -> String {
    format!(
        r#"(func $__rt_fopen (param $path i32) (param $path_len i64) (param $mode i32) (param $mode_len i64) (param $warn i32) (result i32)
  (local $dirfd i32)
  (local $first i32)
  (local $plus i32)
  (local $i i64)
  (local $oflags i32)
  (local $rights i64)
  (local $fdflags i32)
  (local $readable i32)
  (local $writable i32)
  (local.set $first (call $__rt_std_stream_fd (local.get $path) (local.get $path_len)))
  (if (i32.ge_s (local.get $first) (i32.const 0))                ;; a standard stream needs no path
    (then (return (call $__rt_mixed_from_value (i64.const 9)
      (i64.extend_i32_u (local.get $first)) (i64.const 0)))))
  (local.set $first (i32.const 0))
  (local.set $dirfd (call $__rt_wasi_dirfd))
  (if (i32.lt_s (local.get $dirfd) (i32.const 0))                ;; no preopen -> no filesystem
    (then (return (call $__rt_fopen_failed (local.get $warn)))))
  (if (i64.eqz (local.get $mode_len))                            ;; an empty mode opens nothing
    (then (return (call $__rt_fopen_failed (local.get $warn)))))
  (local.set $first (i32.load8_u (local.get $mode)))
  (block $scanned (loop $scan                                    ;; is there a '+' after the first byte?
    (br_if $scanned (i64.ge_u (local.get $i) (local.get $mode_len)))
    (if (i32.eq (i32.load8_u (i32.add (local.get $mode) (i32.wrap_i64 (local.get $i)))) (i32.const 43))
      (then (local.set $plus (i32.const 1)) (br $scanned)))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $scan)))
  (if (i32.eq (local.get $first) (i32.const 114))                ;; 'r': read an existing file
    (then (local.set $readable (i32.const 1)) (local.set $writable (local.get $plus)))
    (else (if (i32.eq (local.get $first) (i32.const 119))         ;; 'w': create, truncate
      (then
        (local.set $writable (i32.const 1)) (local.set $readable (local.get $plus))
        (local.set $oflags (i32.const 9)))                        ;; O_CREAT | O_TRUNC
      (else (if (i32.eq (local.get $first) (i32.const 97))         ;; 'a': create, append
        (then
          (local.set $writable (i32.const 1)) (local.set $readable (local.get $plus))
          (local.set $oflags (i32.const 1))                        ;; O_CREAT
          (local.set $fdflags (i32.const 1)))                      ;; FDFLAG_APPEND
        (else (if (i32.eq (local.get $first) (i32.const 120))       ;; 'x': create, fail if present
          (then
            (local.set $writable (i32.const 1)) (local.set $readable (local.get $plus))
            (local.set $oflags (i32.const 5)))                      ;; O_CREAT | O_EXCL
          (else (if (i32.eq (local.get $first) (i32.const 99))       ;; 'c': create, keep contents
            (then
              (local.set $writable (i32.const 1)) (local.set $readable (local.get $plus))
              (local.set $oflags (i32.const 1)))                     ;; O_CREAT
            (else                                                    ;; anything else is not a PHP mode
              (return (call $__rt_fopen_failed (local.get $warn)))))))))))))
  (local.set $rights (i64.const 2097188))                         ;; FD_SEEK | FD_TELL | FD_FILESTAT_GET
  (if (local.get $readable)
    (then (local.set $rights (i64.or (local.get $rights) (i64.const 2)))))       ;; FD_READ
  (if (local.get $writable)
    (then (local.set $rights (i64.or (local.get $rights) (i64.const 4194369))))) ;; FD_WRITE | FD_DATASYNC | FD_FILESTAT_SET_SIZE
  (if (i32.ne (call $wasi_path_open
        (local.get $dirfd)
        (i32.const 1)                                             ;; LOOKUP_SYMLINK_FOLLOW
        (local.get $path)
        (i32.wrap_i64 (local.get $path_len))
        (local.get $oflags)
        (local.get $rights)
        (local.get $rights)                                       ;; inheriting: same rights
        (local.get $fdflags)
        (i32.add (global.get $__float_scratch) (i32.const {opened})))
      (i32.const 0))
    (then (return (call $__rt_fopen_failed (local.get $warn)))))
  (call $__rt_mixed_from_value (i64.const 9)                      ;; tag 9 = PHP resource
    (i64.extend_i32_u (i32.load (i32.add (global.get $__float_scratch) (i32.const {opened}))))
    (i64.const 0)))
"#,
        opened = IO_SCRATCH + 0x10
    )
}

/// `__rt_stream_fd`: the WASI fd inside a stream handle, or -1 for anything else.
///
/// `fwrite($notAStream, ...)` is a `TypeError` in php-src; here a non-resource cell —
/// including the `false` a failed `fopen` answers — reads as -1 and every caller
/// answers PHP's failure value. That is the native backend's behaviour too.
fn rt_stream_fd() -> String {
    r#"(func $__rt_stream_fd (param $cell i32) (result i32)
  (local $tag i64)
  (local $lo i64)
  (call $__rt_mixed_unbox (local.get $cell))
  (drop)                                                          ;; discard the high word
  (local.set $lo)
  (local.set $tag)
  (if (i64.ne (local.get $tag) (i64.const 9))                     ;; not a resource
    (then (return (i32.const -1))))
  (i32.wrap_i64 (local.get $lo)))
"#
    .to_string()
}

/// `__rt_fwrite`: writes a string to a stream and answers the byte count.
fn rt_fwrite() -> String {
    format!(
        r#"(func $__rt_fwrite (param $fd i32) (param $ptr i32) (param $len i64) (result i64)
  (if (i32.lt_s (local.get $fd) (i32.const 0))
    (then (return (i64.const 0))))
  (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov})) (local.get $ptr))
  (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov_len})) (i32.wrap_i64 (local.get $len)))
  (if (i32.ne (call $wasi_fd_write
        (local.get $fd)
        (i32.add (global.get $__float_scratch) (i32.const {iov}))
        (i32.const 1)
        (i32.add (global.get $__float_scratch) (i32.const {written})))
      (i32.const 0))
    (then (return (i64.const 0))))
  (i64.extend_i32_u (i32.load (i32.add (global.get $__float_scratch) (i32.const {written})))))
"#,
        iov = IO_SCRATCH,
        iov_len = IO_SCRATCH + 4,
        written = IO_SCRATCH + 8
    )
}

/// `__rt_fread`: reads at most `count` bytes and answers them as an owned string.
///
/// The bytes land in a heap block sized to the request and are persisted at the length
/// actually read, so a short read — the normal end-of-file case — answers a string of
/// the right length rather than one padded with whatever the block held. The block is
/// freed once its bytes are copied out, so a read loop costs one live string, not one
/// live block per iteration.
fn rt_fread() -> String {
    format!(
        r#"(func $__rt_fread (param $fd i32) (param $count i64) (result i32 i64)
  (local $buf i32)
  (local $read i32)
  (local $out i32)
  (local $out_len i64)
  (if (i32.or (i32.lt_s (local.get $fd) (i32.const 0)) (i64.le_s (local.get $count) (i64.const 0)))
    (then (return (call $__rt_str_persist (i32.const 0) (i64.const 0)))))
  (local.set $buf (call $__rt_heap_alloc (i32.wrap_i64 (local.get $count))))
  (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov})) (local.get $buf))
  (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov_len})) (i32.wrap_i64 (local.get $count)))
  (if (i32.eqz (call $wasi_fd_read
        (local.get $fd)
        (i32.add (global.get $__float_scratch) (i32.const {iov}))
        (i32.const 1)
        (i32.add (global.get $__float_scratch) (i32.const {nread}))))
    (then (local.set $read (i32.load (i32.add (global.get $__float_scratch) (i32.const {nread}))))))
  (call $__rt_str_persist (local.get $buf) (i64.extend_i32_u (local.get $read)))
  (local.set $out_len)                                            ;; persisted length (on top)
  (local.set $out)                                                ;; persisted pointer
  (call $__rt_heap_free (local.get $buf))                         ;; the bytes are copied now
  (local.get $out)
  (local.get $out_len))
"#,
        iov = IO_SCRATCH,
        iov_len = IO_SCRATCH + 4,
        nread = IO_SCRATCH + 8
    )
}

/// `__rt_fclose`: closes a stream, answering PHP's true/false — but never fd 0, 1 or 2.
///
/// Measured on php-src 8.5.6: `fclose(fopen("php://stdout", "w"))` answers true and leaves
/// stdout working, because that stream wraps the fd without owning it. Here both spellings of a
/// standard stream ARE the fd, so closing it would take the process's output with it — every
/// later `echo` would vanish. Skipping the close keeps the common case exact.
///
/// It diverges for `fclose(STDOUT)` itself, which php-src really does close: there the script
/// loses its output and, measured, dies at the next write. Nothing depends on that, and the
/// alternative failure mode — silently swallowing all remaining output — is far worse.
fn rt_fclose() -> String {
    r#"(func $__rt_fclose (param $fd i32) (result i64)
  (if (i32.lt_s (local.get $fd) (i32.const 0))
    (then (return (i64.const 0))))
  (if (i32.lt_s (local.get $fd) (i32.const 3))                    ;; stdin/stdout/stderr survive
    (then (return (i64.const 1))))
  (if (i32.ne (call $wasi_fd_close (local.get $fd)) (i32.const 0))
    (then (return (i64.const 0))))
  (i64.const 1))
"#
    .to_string()
}

/// `__rt_file_exists`: whether a path resolves, following symlinks.
fn rt_file_exists() -> String {
    format!(
        r#"(func $__rt_file_exists (param $path i32) (param $path_len i64) (result i64)
  (local $dirfd i32)
  (local.set $dirfd (call $__rt_wasi_dirfd))
  (if (i32.lt_s (local.get $dirfd) (i32.const 0))
    (then (return (i64.const 0))))
  (if (i32.ne (call $wasi_path_filestat_get
        (local.get $dirfd)
        (i32.const 1)                                             ;; LOOKUP_SYMLINK_FOLLOW
        (local.get $path)
        (i32.wrap_i64 (local.get $path_len))
        (i32.add (global.get $__float_scratch) (i32.const {stat})))
      (i32.const 0))
    (then (return (i64.const 0))))
  (i64.const 1))
"#,
        stat = IO_SCRATCH + 0x40
    )
}

/// `__rt_unlink`: removes a file, answering PHP's true/false.
fn rt_unlink() -> String {
    r#"(func $__rt_unlink (param $path i32) (param $path_len i64) (result i64)
  (local $dirfd i32)
  (local.set $dirfd (call $__rt_wasi_dirfd))
  (if (i32.lt_s (local.get $dirfd) (i32.const 0))
    (then (return (i64.const 0))))
  (if (i32.ne (call $wasi_path_unlink_file
        (local.get $dirfd)
        (local.get $path)
        (i32.wrap_i64 (local.get $path_len)))
      (i32.const 0))
    (then (return (i64.const 0))))
  (i64.const 1))
"#
    .to_string()
}

/// `__rt_file_size`: a path's size in bytes, or -1 when it does not resolve.
///
/// Read from `path_filestat_get` rather than from the open fd so that sizing a file
/// needs no right beyond the one the preopen already grants for the lookup.
fn rt_file_size() -> String {
    format!(
        r#"(func $__rt_file_size (param $path i32) (param $path_len i64) (result i64)
  (local $dirfd i32)
  (local.set $dirfd (call $__rt_wasi_dirfd))
  (if (i32.lt_s (local.get $dirfd) (i32.const 0))
    (then (return (i64.const -1))))
  (if (i32.ne (call $wasi_path_filestat_get
        (local.get $dirfd)
        (i32.const 1)
        (local.get $path)
        (i32.wrap_i64 (local.get $path_len))
        (i32.add (global.get $__float_scratch) (i32.const {stat})))
      (i32.const 0))
    (then (return (i64.const -1))))
  (i64.load (i32.add (global.get $__float_scratch) (i32.const {size}))))
"#,
        stat = IO_SCRATCH + 0x40,
        size = IO_SCRATCH + 0x40 + 32
    )
}

/// `__rt_file_get_contents`: the whole file as an owned string.
///
/// Sized from the directory stat and then read in a loop, because a single `fd_read`
/// is allowed to answer short and a file read that stops early would silently truncate.
/// A path that does not resolve answers boxed `false`, exactly as PHP does — the EIR
/// types this result `string|false`, so the failure value has somewhere to go.
fn rt_file_get_contents() -> String {
    format!(
        r#"(func $__rt_file_get_contents (param $path i32) (param $path_len i64) (result i32)
  (local $size i64)
  (local $handle i32)
  (local $fd i32)
  (local $buf i32)
  (local $filled i32)
  (local $chunk i32)
  (local $out i32)
  (local.set $size (call $__rt_file_size (local.get $path) (local.get $path_len)))
  (if (i64.lt_s (local.get $size) (i64.const 0))
    (then
      (call $__rt_warn_file_get_contents_failed)                   ;; names ITSELF, not fopen
      (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const {mode})) (i32.const 114))  ;; 'r'
  (local.set $handle (call $__rt_fopen (local.get $path) (local.get $path_len)
    (i32.add (global.get $__float_scratch) (i32.const {mode})) (i64.const 1) (i32.const 0)))
  (local.set $fd (call $__rt_stream_fd (local.get $handle)))
  (if (i32.lt_s (local.get $fd) (i32.const 0))
    (then
      (call $__rt_decref_any (local.get $handle))
      (call $__rt_warn_file_get_contents_failed)
      (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $buf (call $__rt_heap_alloc (i32.wrap_i64 (i64.add (local.get $size) (i64.const 1)))))
  (block $done (loop $more
    (br_if $done (i32.ge_u (local.get $filled) (i32.wrap_i64 (local.get $size))))
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov}))
      (i32.add (local.get $buf) (local.get $filled)))
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov_len}))
      (i32.sub (i32.wrap_i64 (local.get $size)) (local.get $filled)))
    (br_if $done (i32.ne (call $wasi_fd_read
      (local.get $fd)
      (i32.add (global.get $__float_scratch) (i32.const {iov}))
      (i32.const 1)
      (i32.add (global.get $__float_scratch) (i32.const {nread}))) (i32.const 0)))
    (local.set $chunk (i32.load (i32.add (global.get $__float_scratch) (i32.const {nread}))))
    (br_if $done (i32.eqz (local.get $chunk)))                    ;; end of file
    (local.set $filled (i32.add (local.get $filled) (local.get $chunk)))
    (br $more)))
  (drop (call $wasi_fd_close (local.get $fd)))
  (call $__rt_decref_any (local.get $handle))
  (local.set $out (call $__rt_mixed_from_value                    ;; tag 1 persists its own copy
    (i64.const 1) (i64.extend_i32_u (local.get $buf)) (i64.extend_i32_u (local.get $filled))))
  (call $__rt_heap_free (local.get $buf))                         ;; the bytes are copied now
  (local.get $out))
"#,
        iov = IO_SCRATCH,
        iov_len = IO_SCRATCH + 4,
        nread = IO_SCRATCH + 8,
        mode = IO_SCRATCH + 0x30
    )
}

/// `__rt_file_put_contents`: writes a string to a path, answering the byte count.
///
/// Opens in PHP's `w` mode — create, truncate — which is what `file_put_contents`
/// means without `FILE_APPEND`. A path that cannot be opened answers 0, where PHP
/// answers `false` after a warning.
fn rt_file_put_contents() -> String {
    format!(
        r#"(func $__rt_file_put_contents (param $path i32) (param $path_len i64) (param $data i32) (param $data_len i64) (result i64)
  (local $handle i32)
  (local $written i64)
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const {mode})) (i32.const 119))  ;; 'w'
  (local.set $handle (call $__rt_fopen (local.get $path) (local.get $path_len)
    (i32.add (global.get $__float_scratch) (i32.const {mode})) (i64.const 1) (i32.const 0)))
  (if (i32.lt_s (call $__rt_stream_fd (local.get $handle)) (i32.const 0))
    (then
      (call $__rt_decref_any (local.get $handle))
      (return (i64.const 0))))
  (local.set $written (call $__rt_fwrite (call $__rt_stream_fd (local.get $handle)) (local.get $data) (local.get $data_len)))
  (drop (call $__rt_fclose (call $__rt_stream_fd (local.get $handle))))
  (call $__rt_decref_any (local.get $handle))
  (local.get $written))
"#,
        mode = IO_SCRATCH + 0x30
    )
}
