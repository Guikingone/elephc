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
    wm.add_raw_func(RT_IS_MEMSTREAM_PATH);
    wm.add_raw_func(&rt_memstream_new());
    wm.add_raw_func(RT_MEMSTREAM_GROW);
    wm.add_raw_func(&rt_memstream_write());
    wm.add_raw_func(&rt_memstream_read());
    wm.add_raw_func(RT_MEMSTREAM_TELL);
    wm.add_raw_func(RT_MEMSTREAM_SEEK);
    wm.add_raw_func(RT_MEMSTREAM_EOF);
    wm.add_raw_func(RT_MEMSTREAM_CLOSE);
    wm.add_raw_func(RT_FEOF);
    wm.add_raw_func(RT_FTELL);
    wm.add_raw_func(RT_FSEEK);
    wm.add_raw_func(RT_REWIND);
}


/// `__rt_is_memstream_path`: whether a path names an in-memory stream.
///
/// `php://memory` is 13 bytes and `php://temp` 12. php-src also accepts a `php://temp/maxmemory:N`
/// suffix, which only chooses when the stream spills to a real file — something this
/// implementation never does — so the prefix decides and the suffix is ignored.
const RT_IS_MEMSTREAM_PATH: &str = r#"(func $__rt_is_memstream_path (param $p i32) (param $len i64) (result i32)
  (if (i64.lt_u (local.get $len) (i64.const 10))
    (then (return (i32.const 0))))
  (if (i32.eqz (i32.and (i32.and
        (i32.and (i32.eq (i32.load8_u (local.get $p)) (i32.const 112))
                 (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 1))) (i32.const 104)))
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 2))) (i32.const 112))
                 (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 3))) (i32.const 58))))
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 4))) (i32.const 47))
                 (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 5))) (i32.const 47)))))
    (then (return (i32.const 0))))
  (if (i32.and                                                    ;; "temp"
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 6))) (i32.const 116))
                 (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 7))) (i32.const 101)))
        (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 8))) (i32.const 109))
                 (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 9))) (i32.const 112))))
    (then (return (i32.const 1))))
  (if (i64.ne (local.get $len) (i64.const 12))                    ;; "memory" is exactly 12 bytes
    (then (return (i32.const 0))))
  (i32.and
    (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 6))) (i32.const 109))
             (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 7))) (i32.const 101)))
    (i32.and
      (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 8))) (i32.const 109))
               (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 9))) (i32.const 111)))
      (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 10))) (i32.const 114))
               (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 11))) (i32.const 121))))))
"#;

/// `__rt_memstream_write`: writes at the current position, OVERWRITING what is there.
///
/// Measured on php-src 8.5.6: a write mid-stream replaces bytes rather than inserting, and only
/// a write that runs past the end extends the length. `"abcdef"` rewound and written `"XY"`
/// reads back `"XYcdef"`, which is what the length update below preserves.
fn rt_memstream_write() -> String {
    r#"(func $__rt_memstream_write (param $h i32) (param $ptr i32) (param $len i64) (result i64)
  (local $d i32) (local $pos i64) (local $end i64) (local $buf i32) (local $i i64)
  (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
  (if (i64.le_s (local.get $len) (i64.const 0))
    (then (return (i64.const 0))))
  (local.set $pos (i64.load (i32.add (local.get $d) (i32.const 16))))
  (local.set $end (i64.add (local.get $pos) (local.get $len)))
  (call $__rt_memstream_grow (local.get $d) (local.get $end))
  (local.set $buf (i32.load (i32.add (local.get $d) (i32.const 32))))
  (local.set $i (i64.const 0))
  (block $done (loop $copy
    (br_if $done (i64.ge_u (local.get $i) (local.get $len)))
    (i32.store8
      (i32.add (local.get $buf) (i32.wrap_i64 (i64.add (local.get $pos) (local.get $i))))
      (i32.load8_u (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $copy)))
  (i64.store (i32.add (local.get $d) (i32.const 16)) (local.get $end))
  ;; Only a write PAST the end extends the stream; one inside it leaves the length alone.
  (if (i64.gt_u (local.get $end) (i64.load (local.get $d)))
    (then (i64.store (local.get $d) (local.get $end))))
  (local.get $len))
"#
    .to_string()
}

/// `__rt_memstream_read`: reads at most `count` bytes from the current position.
///
/// What sets the end-of-file flag is a read that ASKED for more than was there, not one that
/// merely finished at the end. Measured on php-src 8.5.6 across four cases: requesting 5 of 5
/// leaves `feof` FALSE, requesting 100 of 6 sets it TRUE even though 6 bytes came back, and
/// requesting anything at all of 0 sets it. So the test is `count > available`, and the flag is
/// stored rather than derived from the position, which could not tell those cases apart.
fn rt_memstream_read() -> String {
    r#"(func $__rt_memstream_read (param $h i32) (param $count i64) (result i32 i64)
  (local $d i32) (local $pos i64) (local $len i64) (local $avail i64) (local $take i64)
  (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
  (local.set $pos (i64.load (i32.add (local.get $d) (i32.const 16))))
  (local.set $len (i64.load (local.get $d)))
  (local.set $avail (select
    (i64.sub (local.get $len) (local.get $pos))
    (i64.const 0)
    (i64.lt_u (local.get $pos) (local.get $len))))
  (local.set $take (select (local.get $count) (local.get $avail)
    (i64.lt_u (local.get $count) (local.get $avail))))
  (if (i64.le_s (local.get $count) (i64.const 0))
    (then (local.set $take (i64.const 0))))
  (if (i64.gt_u (local.get $count) (local.get $avail))             ;; asked past the end
    (then (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 1))))
  (if (i64.eqz (local.get $take))
    (then (return (call $__rt_str_persist (i32.const 0) (i64.const 0)))))
  (i64.store (i32.add (local.get $d) (i32.const 16)) (i64.add (local.get $pos) (local.get $take)))
  (call $__rt_str_persist
    (i32.add (i32.load (i32.add (local.get $d) (i32.const 32))) (i32.wrap_i64 (local.get $pos)))
    (local.get $take)))
"#
    .to_string()
}

/// The bit that marks a stream handle as an IN-MEMORY stream rather than a WASI fd.
///
/// `php://memory` and `php://temp` have no host file behind them, so their handles cannot be
/// fds — but they flow through the same resource-tagged cell every other stream does, and the
/// same `fread`/`fwrite`/`fclose` call sites. Encoding the descriptor's ADDRESS with this bit
/// set makes the two spaces disjoint without a side table: a WASI fd is a small non-negative
/// integer, and a linear-memory address never reaches 2^30 in a module this size.
const MEMSTREAM_FLAG: u32 = 0x4000_0000;

/// `__rt_memstream_new`: opens an empty in-memory stream and answers its handle.
///
/// The descriptor is a fixed 40-byte block whose ADDRESS is the handle, so it never moves; the
/// bytes live in a separate block it points at, which is what lets a write grow the stream
/// without invalidating the handle the script is holding.
///
/// Layout: +0 length, +8 capacity, +16 position, +24 eof, +32 buffer pointer.
fn rt_memstream_new() -> String {
    format!(
        r#"(func $__rt_memstream_new (result i32)
  (local $d i32)
  (local.set $d (call $__rt_heap_alloc (i32.const 40)))
  (i64.store (local.get $d) (i64.const 0))                        ;; length
  (i64.store (i32.add (local.get $d) (i32.const 8)) (i64.const 0)) ;; capacity
  (i64.store (i32.add (local.get $d) (i32.const 16)) (i64.const 0)) ;; position
  (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 0)) ;; eof
  (i32.store (i32.add (local.get $d) (i32.const 32)) (i32.const 0)) ;; no buffer yet
  (i32.or (local.get $d) (i32.const {flag})))
"#,
        flag = MEMSTREAM_FLAG
    )
}

/// `__rt_memstream_grow`: ensures the stream's buffer holds at least `want` bytes.
///
/// Capacity doubles from a 32-byte floor, so a write loop costs a logarithmic number of copies
/// rather than one per call. The old bytes are copied and the old block freed; only the buffer
/// pointer in the descriptor changes, which is why the handle stays valid.
const RT_MEMSTREAM_GROW: &str = r#"(func $__rt_memstream_grow (param $d i32) (param $want i64)
  (local $cap i64) (local $old i32) (local $new i32) (local $i i64)
  (local.set $cap (i64.load (i32.add (local.get $d) (i32.const 8))))
  (if (i64.le_u (local.get $want) (local.get $cap))
    (then (return)))
  (if (i64.lt_u (local.get $cap) (i64.const 32))
    (then (local.set $cap (i64.const 32))))
  (block $sized (loop $double
    (br_if $sized (i64.ge_u (local.get $cap) (local.get $want)))
    (local.set $cap (i64.shl (local.get $cap) (i64.const 1)))
    (br $double)))
  (local.set $old (i32.load (i32.add (local.get $d) (i32.const 32))))
  (local.set $new (call $__rt_heap_alloc (i32.wrap_i64 (local.get $cap))))
  (local.set $i (i64.const 0))
  (block $copied (loop $copy
    (br_if $copied (i64.ge_u (local.get $i) (i64.load (local.get $d))))  ;; only the LIVE bytes
    (i32.store8
      (i32.add (local.get $new) (i32.wrap_i64 (local.get $i)))
      (i32.load8_u (i32.add (local.get $old) (i32.wrap_i64 (local.get $i)))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $copy)))
  (if (local.get $old)
    (then (call $__rt_heap_free (local.get $old))))
  (i32.store (i32.add (local.get $d) (i32.const 32)) (local.get $new))
  (i64.store (i32.add (local.get $d) (i32.const 8)) (local.get $cap)))
"#;

/// `__rt_memstream_tell`: the stream's current position.
const RT_MEMSTREAM_TELL: &str = r#"(func $__rt_memstream_tell (param $h i32) (result i64)
  (i64.load (i32.add (i32.and (local.get $h) (i32.const 1073741823)) (i32.const 16))))
"#;

/// `__rt_memstream_seek`: moves the position and clears the end-of-file flag.
///
/// `rewind` is this with offset 0. Clearing `eof` is what php-src does: after `rewind`, `feof`
/// answers false again even though a read had previously hit the end.
const RT_MEMSTREAM_SEEK: &str = r#"(func $__rt_memstream_seek (param $h i32) (param $off i64)
  (local $d i32)
  (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
  (i64.store (i32.add (local.get $d) (i32.const 16)) (local.get $off))
  (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 0)))
"#;

/// `__rt_memstream_eof`: whether a read has already found nothing.
///
/// Measured on php-src 8.5.6, this is NOT "the position is at the end": reading exactly the last
/// byte leaves `feof` FALSE, and only the next read — the one that finds nothing — sets it. So
/// the flag is written by the read, never derived from the position here.
const RT_MEMSTREAM_EOF: &str = r#"(func $__rt_memstream_eof (param $h i32) (result i64)
  (i64.load (i32.add (i32.and (local.get $h) (i32.const 1073741823)) (i32.const 24))))
"#;

/// `__rt_memstream_close`: frees the buffer and the descriptor, answering PHP's true.
const RT_MEMSTREAM_CLOSE: &str = r#"(func $__rt_memstream_close (param $h i32) (result i64)
  (local $d i32) (local $buf i32)
  (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
  (local.set $buf (i32.load (i32.add (local.get $d) (i32.const 32))))
  (if (local.get $buf)
    (then (call $__rt_heap_free (local.get $buf))))
  (call $__rt_heap_free (local.get $d))
  (i64.const 1))
"#;

/// `__rt_feof`: PHP's `feof` for either kind of stream.
///
/// A WASI fd has no cheap "have we read past the end" bit, so it answers false — which is what
/// the native backend does for a stream it cannot ask. An in-memory stream carries the flag its
/// own reads set.
const RT_FEOF: &str = r#"(func $__rt_feof (param $h i32) (result i64)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (i64.const 1))))                                ;; not a stream at all
  (if (i32.and (local.get $h) (i32.const 1073741824))
    (then (return (call $__rt_memstream_eof (local.get $h)))))
  (i64.const 0))
"#;

/// `__rt_ftell`: PHP's `ftell`, for either stream kind.
///
/// WASI has no `fd_tell`, but a ZERO-length seek from the current position answers the same
/// question and is what the position is defined as. Measured on php-src 8.5.6: after writing
/// ten bytes, `ftell` is 10 — so answering a constant 0 for a real fd, as this used to, was
/// wrong for every file that had been read or written.
const RT_FTELL: &str = r#"(func $__rt_ftell (param $h i32) (result i64)
  (if (i32.and (local.get $h) (i32.const 1073741824))
    (then (return (call $__rt_memstream_tell (local.get $h)))))
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (i64.const 0))))
  (if (i32.ne (call $wasi_fd_seek (local.get $h) (i64.const 0) (i32.const 1)
        (i32.add (global.get $__float_scratch) (i32.const 12352)))
      (i32.const 0))
    (then (return (i64.const 0))))
  (i64.load (i32.add (global.get $__float_scratch) (i32.const 12352))))
"#;

/// `__rt_fseek`: PHP's `fseek`, answering 0 on success and -1 on failure.
///
/// PHP's `SEEK_SET`/`SEEK_CUR`/`SEEK_END` are 0/1/2, which are the same numbers WASI's
/// `fd_seek` uses, so the whence passes straight through for a real fd.
///
/// Measured on php-src 8.5.6, for the two cases that are not obvious:
///   * a NEGATIVE absolute position fails (`-1`) and leaves the position UNTOUCHED;
///   * seeking PAST the end succeeds — `fseek($m, 50)` answers 0 and `ftell` is then 50 —
///     and the read there yields `""`.
/// A successful seek also CLEARS eof, which `__rt_memstream_seek` already does.
/// The whence arrives as an `i64` because the lowering passes PHP ints as i64; `fd_seek`
/// takes it as an i32, so it is narrowed at the call rather than in the signature.
const RT_FSEEK: &str = r#"(func $__rt_fseek (param $h i32) (param $off i64) (param $whence i64) (result i64)
  (local $d i32) (local $target i64)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (i64.const -1))))                                 ;; not a stream at all
  (if (i32.and (local.get $h) (i32.const 1073741824))
    (then
      (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
      (local.set $target
        (if (result i64) (i64.eq (local.get $whence) (i64.const 1))
          (then (i64.add (i64.load (i32.add (local.get $d) (i32.const 16))) (local.get $off)))
          (else (if (result i64) (i64.eq (local.get $whence) (i64.const 2))
            (then (i64.add (i64.load (local.get $d)) (local.get $off)))
            (else (local.get $off))))))
      (if (i64.lt_s (local.get $target) (i64.const 0))
        (then (return (i64.const -1))))                             ;; position left untouched
      (call $__rt_memstream_seek (local.get $h) (local.get $target))
      (return (i64.const 0))))
  (if (i32.ne (call $wasi_fd_seek (local.get $h) (local.get $off)
        (i32.wrap_i64 (local.get $whence))
        (i32.add (global.get $__float_scratch) (i32.const 12352)))
      (i32.const 0))
    (then (return (i64.const -1))))
  (i64.const 0))
"#;

/// `__rt_rewind`: PHP's `rewind`, seeking either kind of stream back to the start.
const RT_REWIND: &str = r#"(func $__rt_rewind (param $h i32) (result i64)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (i64.const 0))))
  (if (i32.and (local.get $h) (i32.const 1073741824))
    (then
      (call $__rt_memstream_seek (local.get $h) (i64.const 0))
      (return (i64.const 1))))
  (if (i32.ne (call $wasi_fd_seek (local.get $h) (i64.const 0) (i32.const 0)
        (i32.add (global.get $__float_scratch) (i32.const 12352)))
      (i32.const 0))
    (then (return (i64.const 0))))
  (i64.const 1))
"#;

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
  ;; `php://memory` and `php://temp` have no host file behind them at all, so they are opened
  ;; before the preopen probe: they work with no filesystem authority whatsoever.
  (if (call $__rt_is_memstream_path (local.get $path) (local.get $path_len))
    (then (return (call $__rt_mixed_from_value (i64.const 9)
      (i64.extend_i32_u (call $__rt_memstream_new)) (i64.const 0)))))
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
  (if (i32.and (local.get $fd) (i32.const {flag}))
    (then (return (call $__rt_memstream_write (local.get $fd) (local.get $ptr) (local.get $len)))))
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
        written = IO_SCRATCH + 8,
        flag = MEMSTREAM_FLAG
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
  (if (i32.and (local.get $fd) (i32.const {flag}))
    (then (return (call $__rt_memstream_read (local.get $fd) (local.get $count)))))
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
        nread = IO_SCRATCH + 8,
        flag = MEMSTREAM_FLAG
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
  (if (i32.and (local.get $fd) (i32.const 1073741824))
    (then (return (call $__rt_memstream_close (local.get $fd)))))
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
