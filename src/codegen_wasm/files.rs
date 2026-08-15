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

/// Offset of the per-fd EOF flag table: one byte per real WASI fd, 256 fds.
///
/// PHP's `feof` is a FLAG a read sets when it finds nothing, never a position
/// comparison — reading exactly the last byte leaves it false, and only the next
/// read flips it (measured on php-src 8.5.6, same rule `__rt_memstream_eof`
/// documents). A real fd carries no descriptor block to hold that flag, so it
/// lives here, indexed by fd. Linear memory starts zeroed, so every fd begins
/// "not at EOF"; `fclose` clears the byte because WASI recycles fd numbers.
const FD_EOF_FLAGS: u32 = IO_SCRATCH + 0x50;

/// Offset of the per-fd stream-metadata table: 16 bytes per fd, 256 fds.
///
/// `stream_get_meta_data` reports the MODE and URI `fopen` was called with, and a
/// bare WASI fd remembers neither — so `fopen` records them here as persisted-string
/// (ptr, len) pairs: mode at +0/+4, uri at +8/+12. `fclose` releases both strings
/// and zeroes the record, because WASI recycles fd numbers. A never-recorded fd
/// (a stream this runtime did not open) reads as two empty strings.
const FD_STREAM_META: u32 = 0x4000;

/// Adds the file-I/O runtime to `wm`. Requires the WASI path imports and the heap
/// and string runtimes, all of which the command runtime emits alongside it.
pub(super) fn emit_file_runtime(wm: &mut WatModule) {
    wm.add_raw_func(RT_FOPEN_FAILED);
    wm.add_raw_func(RT_STD_STREAM_FD);
    wm.add_raw_func(&rt_wasi_dirfd());
    wm.add_raw_func(&rt_fopen());
    wm.add_raw_func(&rt_stream_fd());
    wm.add_raw_func(&rt_fwrite());
    wm.add_raw_func(RT_FWRITE_BOXED);
    wm.add_raw_func(&rt_fread());
    wm.add_raw_func(&rt_fclose());
    wm.add_raw_func(&rt_file_exists());
    wm.add_raw_func(&rt_unlink());
    wm.add_raw_func(&rt_file_size());
    wm.add_raw_func(&rt_file_get_contents());
    wm.add_raw_func(&rt_file_put_contents());
    wm.add_raw_func(RT_IS_MEMSTREAM_PATH);
    wm.add_raw_func(RT_IS_DATA_URI);
    wm.add_raw_func(RT_DATA_URI_OPEN);
    wm.add_raw_func(&rt_memstream_new());
    wm.add_raw_func(RT_MEMSTREAM_GROW);
    wm.add_raw_func(&rt_memstream_write());
    wm.add_raw_func(&rt_memstream_read());
    wm.add_raw_func(RT_MEMSTREAM_TELL);
    wm.add_raw_func(RT_MEMSTREAM_SEEK);
    wm.add_raw_func(RT_MEMSTREAM_EOF);
    wm.add_raw_func(RT_MEMSTREAM_CLOSE);
    wm.add_raw_func(&rt_fd_eof_get());
    wm.add_raw_func(&rt_fd_eof_set());
    wm.add_raw_func(&rt_fd_eof_clear());
    wm.add_raw_func(&rt_fd_meta_record());
    wm.add_raw_func(&rt_fd_meta_clear());
    wm.add_raw_func(RT_FEOF);
    wm.add_raw_func(RT_FTELL);
    wm.add_raw_func(RT_FSEEK);
    wm.add_raw_func(&rt_stream_get_contents());
    wm.add_raw_func(RT_STREAM_COPY_TO_STREAM);
    wm.add_raw_func(RT_REWIND);
    wm.add_raw_func(&rt_stream_get_line());
}

/// `__rt_fd_meta_record`: remembers a real fd's fopen MODE and URI for
/// `stream_get_meta_data`, releasing any stale record the recycled fd left behind.
fn rt_fd_meta_record() -> String {
    format!(
        r#"(func $__rt_fd_meta_record (param $fd i32) (param $mode i32) (param $mode_len i64) (param $uri i32) (param $uri_len i64)
  (local $slot i32) (local $p i32) (local $l i64)
  (if (i32.ge_u (local.get $fd) (i32.const 256))
    (then (return)))
  (call $__rt_fd_meta_clear (local.get $fd))
  (local.set $slot (i32.add (i32.add (global.get $__float_scratch) (i32.const {meta}))
                            (i32.shl (local.get $fd) (i32.const 4))))
  (call $__rt_str_persist (local.get $mode) (local.get $mode_len))
  (local.set $l)
  (local.set $p)
  (i32.store (local.get $slot) (local.get $p))
  (i32.store (i32.add (local.get $slot) (i32.const 4)) (i32.wrap_i64 (local.get $l)))
  (call $__rt_str_persist (local.get $uri) (local.get $uri_len))
  (local.set $l)
  (local.set $p)
  (i32.store (i32.add (local.get $slot) (i32.const 8)) (local.get $p))
  (i32.store (i32.add (local.get $slot) (i32.const 12)) (i32.wrap_i64 (local.get $l))))
"#,
        meta = FD_STREAM_META
    )
}

/// `__rt_fd_meta_clear`: releases a closed fd's recorded mode/uri strings and zeroes
/// the record — the next `fopen` may be handed the same fd number back.
fn rt_fd_meta_clear() -> String {
    format!(
        r#"(func $__rt_fd_meta_clear (param $fd i32)
  (local $slot i32)
  (if (i32.ge_u (local.get $fd) (i32.const 256))
    (then (return)))
  (local.set $slot (i32.add (i32.add (global.get $__float_scratch) (i32.const {meta}))
                            (i32.shl (local.get $fd) (i32.const 4))))
  (call $__rt_decref_any (i32.load (local.get $slot)))
  (call $__rt_decref_any (i32.load (i32.add (local.get $slot) (i32.const 8))))
  (i64.store (local.get $slot) (i64.const 0))
  (i64.store (i32.add (local.get $slot) (i32.const 8)) (i64.const 0)))
"#,
        meta = FD_STREAM_META
    )
}

/// `__rt_fd_eof_get`: the EOF flag a real fd's reads have set, 0 for out-of-table fds.
fn rt_fd_eof_get() -> String {
    format!(
        r#"(func $__rt_fd_eof_get (param $fd i32) (result i64)
  (if (i32.ge_u (local.get $fd) (i32.const 256))
    (then (return (i64.const 0))))
  (i64.extend_i32_u (i32.load8_u
    (i32.add (i32.add (global.get $__float_scratch) (i32.const {eof})) (local.get $fd)))))
"#,
        eof = FD_EOF_FLAGS
    )
}

/// `__rt_fd_eof_set`: records that a read on this fd found nothing.
fn rt_fd_eof_set() -> String {
    format!(
        r#"(func $__rt_fd_eof_set (param $fd i32)
  (if (i32.ge_u (local.get $fd) (i32.const 256))
    (then (return)))
  (i32.store8
    (i32.add (i32.add (global.get $__float_scratch) (i32.const {eof})) (local.get $fd))
    (i32.const 1)))
"#,
        eof = FD_EOF_FLAGS
    )
}

/// `__rt_fd_eof_clear`: forgets the flag — a successful seek does, and `fclose` must,
/// because the next `fopen` may be handed the same fd number back.
fn rt_fd_eof_clear() -> String {
    format!(
        r#"(func $__rt_fd_eof_clear (param $fd i32)
  (if (i32.ge_u (local.get $fd) (i32.const 256))
    (then (return)))
  (i32.store8
    (i32.add (i32.add (global.get $__float_scratch) (i32.const {eof})) (local.get $fd))
    (i32.const 0)))
"#,
        eof = FD_EOF_FLAGS
    )
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
/// The descriptor is a fixed 56-byte block whose ADDRESS is the handle, so it never moves; the
/// bytes live in a separate block it points at, which is what lets a write grow the stream
/// without invalidating the handle the script is holding.
///
/// Layout: +0 length, +8 capacity, +16 position, +24 eof, +32 buffer pointer, then the
/// `stream_get_meta_data` record — +40/+44 the fopen MODE as a persisted-string (ptr, len)
/// and +48/+52 the URI the stream was opened as, both released by `__rt_memstream_close`.
fn rt_memstream_new() -> String {
    format!(
        r#"(func $__rt_memstream_new (param $mode i32) (param $mode_len i64) (param $uri i32) (param $uri_len i64) (result i32)
  (local $d i32) (local $p i32) (local $l i64)
  (local.set $d (call $__rt_heap_alloc (i32.const 56)))
  (i64.store (local.get $d) (i64.const 0))                        ;; length
  (i64.store (i32.add (local.get $d) (i32.const 8)) (i64.const 0)) ;; capacity
  (i64.store (i32.add (local.get $d) (i32.const 16)) (i64.const 0)) ;; position
  (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 0)) ;; eof
  (i32.store (i32.add (local.get $d) (i32.const 32)) (i32.const 0)) ;; no buffer yet
  (call $__rt_str_persist (local.get $mode) (local.get $mode_len))
  (local.set $l)
  (local.set $p)
  (i32.store (i32.add (local.get $d) (i32.const 40)) (local.get $p))               ;; mode ptr
  (i32.store (i32.add (local.get $d) (i32.const 44)) (i32.wrap_i64 (local.get $l))) ;; mode len
  (call $__rt_str_persist (local.get $uri) (local.get $uri_len))
  (local.set $l)
  (local.set $p)
  (i32.store (i32.add (local.get $d) (i32.const 48)) (local.get $p))               ;; uri ptr
  (i32.store (i32.add (local.get $d) (i32.const 52)) (i32.wrap_i64 (local.get $l))) ;; uri len
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

/// `__rt_memstream_close`: frees the buffer, the metadata strings, and the
/// descriptor, answering PHP's true.
const RT_MEMSTREAM_CLOSE: &str = r#"(func $__rt_memstream_close (param $h i32) (result i64)
  (local $d i32) (local $buf i32)
  (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
  (local.set $buf (i32.load (i32.add (local.get $d) (i32.const 32))))
  (if (local.get $buf)
    (then (call $__rt_heap_free (local.get $buf))))
  (call $__rt_decref_any (i32.load (i32.add (local.get $d) (i32.const 40))))  ;; mode string (null-safe)
  (call $__rt_decref_any (i32.load (i32.add (local.get $d) (i32.const 48))))  ;; uri string (null-safe)
  (call $__rt_heap_free (local.get $d))
  (i64.const 1))
"#;

/// `__rt_feof`: PHP's `feof` for either kind of stream.
///
/// An in-memory stream carries the flag its own reads set; a real fd carries the same
/// read-set flag in the `FD_EOF_FLAGS` table. Neither is a position comparison: the
/// stream-lines corpus example needs the read AFTER the last line to run — it answers
/// `false` and only then does `feof` turn true, exactly php's fourth `event:` line.
const RT_FEOF: &str = r#"(func $__rt_feof (param $h i32) (result i64)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (i64.const 1))))                                ;; not a stream at all
  (if (i32.and (local.get $h) (i32.const 1073741824))
    (then (return (call $__rt_memstream_eof (local.get $h)))))
  (call $__rt_fd_eof_get (local.get $h)))
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
  (call $__rt_fd_eof_clear (local.get $h))                          ;; a successful seek clears eof
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
  (call $__rt_fd_eof_clear (local.get $h))                          ;; a successful seek clears eof
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
    (then
      (call $__rt_fd_meta_record (local.get $first)
        (local.get $mode) (local.get $mode_len)
        (local.get $path) (local.get $path_len))
      (return (call $__rt_mixed_from_value (i64.const 9)
        (i64.extend_i32_u (local.get $first)) (i64.const 0)))))
  ;; `php://memory` and `php://temp` have no host file behind them at all, so they are opened
  ;; before the preopen probe: they work with no filesystem authority whatsoever.
  (if (call $__rt_is_memstream_path (local.get $path) (local.get $path_len))
    (then (return (call $__rt_mixed_from_value (i64.const 9)
      (i64.extend_i32_u (call $__rt_memstream_new
        (local.get $mode) (local.get $mode_len)
        (local.get $path) (local.get $path_len))) (i64.const 0)))))
  ;; A `data:` URI carries its own content, so it is decoded here rather than opened: no
  ;; filesystem authority is involved at all. A malformed one answers php's `false`.
  (if (call $__rt_is_data_uri (local.get $path) (local.get $path_len))
    (then
      (local.set $first (call $__rt_data_uri_open (local.get $path) (local.get $path_len)
                                                  (local.get $mode) (local.get $mode_len)))
      (if (i32.lt_s (local.get $first) (i32.const 0))
        (then (return (call $__rt_fopen_failed (local.get $warn)))))
      (return (call $__rt_mixed_from_value (i64.const 9)
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
  (call $__rt_fd_meta_record
    (i32.load (i32.add (global.get $__float_scratch) (i32.const {opened})))
    (local.get $mode) (local.get $mode_len)
    (local.get $path) (local.get $path_len))
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
/// `__rt_fwrite_boxed`: `fwrite`'s byte count boxed as PHP's `int|false` cell.
///
/// The EIR types the result `mixed` since upstream widened the contract. This runtime
/// never answers `false` — a failed write reports 0 bytes, as the raw helper always
/// did — so the box always carries tag 0. The cell is what the consumer releases.
const RT_FWRITE_BOXED: &str = r#"(func $__rt_fwrite_boxed (param $fd i32) (param $ptr i32) (param $len i64) (result i32)
  (call $__rt_mixed_from_value (i64.const 0)
    (call $__rt_fwrite (local.get $fd) (local.get $ptr) (local.get $len))
    (i64.const 0)))
"#;

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
  (if (i32.eqz (local.get $read))                                 ;; the read that finds nothing sets EOF
    (then (call $__rt_fd_eof_set (local.get $fd))))
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

/// `__rt_stream_get_contents`: the remainder of a stream, boxed as PHP's `string|false`.
///
/// The bytes are gathered into ONE raw block and boxed once. Reusing `__rt_fread` would be
/// shorter but would persist the string twice — once in the read and once in the boxing — and
/// leak the first copy on every call.
///
/// The length is resolved BEFORE reading rather than by concatenating in a loop: an in-memory
/// stream knows its own remaining count, and a real fd answers it with a seek to the end and
/// back. Measured on php-src 8.5.6 against `examples/stream-get-contents`: a whole-file read,
/// a read that resumes from the current position after a partial `fread`, a capped read, and a
/// read from an explicit offset all agree.
fn rt_stream_get_contents() -> String {
    format!(
        r#"(func $__rt_stream_get_contents (param $h i32) (param $maxlen i64) (param $offset i64) (result i32)
  (local $d i32) (local $pos i64) (local $avail i64) (local $count i64)
  (local $cur i64) (local $end i64) (local $buf i32) (local $filled i32) (local $chunk i32)
  (local $out i32)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (if (i64.ge_s (local.get $offset) (i64.const 0))                ;; -1 means "read from here"
    (then (drop (call $__rt_fseek (local.get $h) (local.get $offset) (i64.const 0)))))
  (if (i32.and (local.get $h) (i32.const {flag}))
    (then
      (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
      (local.set $pos (i64.load (i32.add (local.get $d) (i32.const 16))))
      (local.set $avail (i64.sub (i64.load (local.get $d)) (local.get $pos))))
    (else
      (local.set $cur (call $__rt_ftell (local.get $h)))
      (drop (call $__rt_fseek (local.get $h) (i64.const 0) (i64.const 2)))
      (local.set $end (call $__rt_ftell (local.get $h)))
      (drop (call $__rt_fseek (local.get $h) (local.get $cur) (i64.const 0)))
      (local.set $avail (i64.sub (local.get $end) (local.get $cur)))))
  (if (i64.lt_s (local.get $avail) (i64.const 0))                 ;; seeked past the end
    (then (local.set $avail (i64.const 0))))
  (local.set $count (local.get $avail))
  (if (i32.and (i64.ge_s (local.get $maxlen) (i64.const 0))
               (i64.lt_s (local.get $maxlen) (local.get $avail)))
    (then (local.set $count (local.get $maxlen))))
  (if (i64.le_s (local.get $count) (i64.const 0))
    (then (return (call $__rt_mixed_from_value (i64.const 1) (i64.const 0) (i64.const 0)))))
  (if (i32.and (local.get $h) (i32.const {flag}))
    (then
      (local.set $buf (i32.add (i32.load (i32.add (local.get $d) (i32.const 32)))
                               (i32.wrap_i64 (local.get $pos))))
      (local.set $out (call $__rt_mixed_from_value (i64.const 1)
        (i64.extend_i32_u (local.get $buf)) (local.get $count)))
      (i64.store (i32.add (local.get $d) (i32.const 16))          ;; the read advances the stream
        (i64.add (local.get $pos) (local.get $count)))
      (return (local.get $out))))
  (local.set $buf (call $__rt_heap_alloc (i32.wrap_i64 (local.get $count))))
  (block $done (loop $more
    (br_if $done (i32.ge_u (local.get $filled) (i32.wrap_i64 (local.get $count))))
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov}))
      (i32.add (local.get $buf) (local.get $filled)))
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov_len}))
      (i32.sub (i32.wrap_i64 (local.get $count)) (local.get $filled)))
    (br_if $done (i32.ne (call $wasi_fd_read
      (local.get $h)
      (i32.add (global.get $__float_scratch) (i32.const {iov}))
      (i32.const 1)
      (i32.add (global.get $__float_scratch) (i32.const {nread}))) (i32.const 0)))
    (local.set $chunk (i32.load (i32.add (global.get $__float_scratch) (i32.const {nread}))))
    (if (i32.eqz (local.get $chunk))                              ;; a short read means the end
      (then
        (call $__rt_fd_eof_set (local.get $h))                    ;; and the read that found it sets eof
        (br $done)))
    (local.set $filled (i32.add (local.get $filled) (local.get $chunk)))
    (br $more)))
  (local.set $out (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (local.get $buf)) (i64.extend_i32_u (local.get $filled))))
  (call $__rt_heap_free (local.get $buf))                         ;; the bytes are copied now
  (local.get $out))
"#,
        iov = IO_SCRATCH,
        iov_len = IO_SCRATCH + 4,
        nread = IO_SCRATCH + 8,
        flag = MEMSTREAM_FLAG
    )
}

/// `__rt_stream_copy_to_stream`: pipes a stream's remainder into another, boxed as `int|false`.
///
/// Composed from the two helpers rather than reimplementing a read loop: the source read is
/// exactly `stream_get_contents`, including how it resolves "everything remaining" and honours
/// an explicit offset, and the destination write is `fwrite`. Reading the whole span at once
/// matches what `stream_get_contents` already does and keeps one contract for both stream kinds.
///
/// The intermediate cell is released here — it is this function's own reference, and the string
/// bytes have been written out by the time it goes.
const RT_STREAM_COPY_TO_STREAM: &str = r#"(func $__rt_stream_copy_to_stream (param $from i32) (param $to i32) (param $len i64) (param $offset i64) (result i32)
  (local $cell i32) (local $ptr i32) (local $n i64) (local $written i64)
  (local.set $cell (call $__rt_stream_get_contents (local.get $from) (local.get $len) (local.get $offset)))
  (if (i64.ne (i64.load (local.get $cell)) (i64.const 1))         ;; tag 1 = string; anything else failed
    (then
      (call $__rt_decref_any (local.get $cell))
      (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $ptr (i32.wrap_i64 (i64.load (i32.add (local.get $cell) (i32.const 8)))))
  (local.set $n (i64.load (i32.add (local.get $cell) (i32.const 16))))
  (local.set $written (call $__rt_fwrite (local.get $to) (local.get $ptr) (local.get $n)))
  (call $__rt_decref_any (local.get $cell))
  (call $__rt_mixed_from_value (i64.const 0) (local.get $written) (i64.const 0)))
"#;

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
  (call $__rt_fd_eof_clear (local.get $fd))                       ;; the next fopen may reuse this fd number
  (call $__rt_fd_meta_clear (local.get $fd))                      ;; and must not inherit this stream's mode/uri
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

/// Emits `__rt_stream_get_meta_data`, whose nine keys and four constant values live in
/// the command data region — which is why it is emitted from the failure runtime's
/// tail, where their offsets are known, and not from `emit_file_runtime`.
///
/// `offsets` is the 14-entry (offset, len) slice laid out by `emit_failure_runtime`:
/// the nine keys in php's own insertion order (timed_out, blocked, eof, wrapper_type,
/// stream_type, mode, unread_bytes, seekable, uri) then plainfile, STDIO, PHP, MEMORY,
/// TEMP.
///
/// Measured on php-src 8.5.6:
///   * a real file answers wrapper_type "plainfile" / stream_type "STDIO", the MODE
///     exactly as `fopen` received it ("rb" stays "rb"), and the URI as given;
///   * `php://memory` and `php://temp` answer "PHP" with "MEMORY"/"TEMP", and their
///     mode is NORMALIZED to binary — "r" reports "rb", "w+" reports "w+b";
///   * `blocked` is true and `timed_out` false for both; `unread_bytes` is 0;
///   * `seekable` is asked of the fd itself here — a zero-length SEEK_CUR succeeds
///     only on a seekable fd, so pipes and ttys answer false the way php does.
///
/// A stream this runtime never recorded (a bare std fd) reads as two empty strings
/// for mode/uri rather than fabricated values.
pub(super) fn emit_stream_meta_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    let [k_to, k_bl, k_eof, k_wt, k_st, k_mode, k_ub, k_seek, k_uri, v_plain, v_stdio, v_php, v_mem, v_temp] =
        offsets
    else {
        unreachable!("emit_failure_runtime hands exactly the fourteen meta fragments");
    };
    wm.add_raw_func(&format!(
        r#"(func $__rt_stream_get_meta_data (param $h i32) (result i32)
  (local $hash i32) (local $d i32) (local $slot i32) (local $eof i64)
  (local $mode_p i32) (local $mode_l i64) (local $uri_p i32) (local $uri_l i64)
  (local $wrap_p i32) (local $wrap_l i64) (local $st_p i32) (local $st_l i64)
  (local $seek i64) (local $mb i32) (local $i i64) (local $hasb i32) (local $out i32)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (if (i32.and (local.get $h) (i32.const {flag}))
    (then
      ;; In-memory stream: the descriptor carries everything.
      (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
      (local.set $eof (i64.extend_i32_u
        (i64.ne (i64.load (i32.add (local.get $d) (i32.const 24))) (i64.const 0))))
      (local.set $mode_p (i32.load (i32.add (local.get $d) (i32.const 40))))
      (local.set $mode_l (i64.extend_i32_u (i32.load (i32.add (local.get $d) (i32.const 44)))))
      (local.set $uri_p (i32.load (i32.add (local.get $d) (i32.const 48))))
      (local.set $uri_l (i64.extend_i32_u (i32.load (i32.add (local.get $d) (i32.const 52)))))
      (local.set $wrap_p (i32.const {php_p}))
      (local.set $wrap_l (i64.const {php_l}))
      ;; The two memstream uris differ in LENGTH: "php://temp" is 10, "php://memory" 12.
      (if (i64.eq (local.get $uri_l) (i64.const 10))
        (then
          (local.set $st_p (i32.const {temp_p}))
          (local.set $st_l (i64.const {temp_l})))
        (else
          (local.set $st_p (i32.const {mem_p}))
          (local.set $st_l (i64.const {mem_l}))))
      (local.set $seek (i64.const 1))
      ;; php reports a memory stream's mode in BINARY: append 'b' when absent.
      (local.set $hasb (i32.const 0))
      (local.set $i (i64.const 0))
      (block $scanned (loop $scan
        (br_if $scanned (i64.ge_u (local.get $i) (local.get $mode_l)))
        (if (i32.eq (i32.load8_u (i32.add (local.get $mode_p) (i32.wrap_i64 (local.get $i)))) (i32.const 98))
          (then
            (local.set $hasb (i32.const 1))
            (br $scanned)))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $scan)))
      (if (i32.eqz (local.get $hasb))
        (then
          (local.set $mb (call $__rt_heap_alloc (i32.add (i32.wrap_i64 (local.get $mode_l)) (i32.const 1))))
          (local.set $i (i64.const 0))
          (block $copied (loop $copy
            (br_if $copied (i64.ge_u (local.get $i) (local.get $mode_l)))
            (i32.store8
              (i32.add (local.get $mb) (i32.wrap_i64 (local.get $i)))
              (i32.load8_u (i32.add (local.get $mode_p) (i32.wrap_i64 (local.get $i)))))
            (local.set $i (i64.add (local.get $i) (i64.const 1)))
            (br $copy)))
          (i32.store8 (i32.add (local.get $mb) (i32.wrap_i64 (local.get $mode_l))) (i32.const 98))
          (local.set $mode_p (local.get $mb))
          (local.set $mode_l (i64.add (local.get $mode_l) (i64.const 1))))))
    (else
      ;; Real fd: the fopen record plus the live eof flag and a seekability probe.
      (local.set $slot (i32.add (i32.add (global.get $__float_scratch) (i32.const {meta}))
                                (i32.shl (local.get $h) (i32.const 4))))
      (if (i32.ge_u (local.get $h) (i32.const 256))
        (then (local.set $slot (i32.add (global.get $__float_scratch) (i32.const {meta})))))  ;; out-of-table fd reads slot 0 (never recorded)
      (local.set $mode_p (i32.load (local.get $slot)))
      (local.set $mode_l (i64.extend_i32_u (i32.load (i32.add (local.get $slot) (i32.const 4)))))
      (local.set $uri_p (i32.load (i32.add (local.get $slot) (i32.const 8))))
      (local.set $uri_l (i64.extend_i32_u (i32.load (i32.add (local.get $slot) (i32.const 12)))))
      (local.set $eof (call $__rt_fd_eof_get (local.get $h)))
      (local.set $wrap_p (i32.const {plain_p}))
      (local.set $wrap_l (i64.const {plain_l}))
      (local.set $st_p (i32.const {stdio_p}))
      (local.set $st_l (i64.const {stdio_l}))
      (local.set $seek (i64.extend_i32_u (i32.eqz (call $wasi_fd_seek
        (local.get $h) (i64.const 0) (i32.const 1)
        (i32.add (global.get $__float_scratch) (i32.const 12352))))))))
  (local.set $hash (call $__rt_hash_new (i64.const 32) (i64.const 7)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_to_p}) (i64.const {k_to_l}) (i64.const 0) (i64.const 0) (i64.const 3)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_bl_p}) (i64.const {k_bl_l}) (i64.const 1) (i64.const 0) (i64.const 3)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_eof_p}) (i64.const {k_eof_l}) (local.get $eof) (i64.const 0) (i64.const 3)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_wt_p}) (i64.const {k_wt_l})
    (i64.extend_i32_u (local.get $wrap_p)) (local.get $wrap_l) (i64.const 1)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_st_p}) (i64.const {k_st_l})
    (i64.extend_i32_u (local.get $st_p)) (local.get $st_l) (i64.const 1)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_mode_p}) (i64.const {k_mode_l})
    (i64.extend_i32_u (local.get $mode_p)) (local.get $mode_l) (i64.const 1)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_ub_p}) (i64.const {k_ub_l}) (i64.const 0) (i64.const 0) (i64.const 0)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_seek_p}) (i64.const {k_seek_l}) (local.get $seek) (i64.const 0) (i64.const 3)))
  (local.set $hash (call $__rt_hash_set (local.get $hash)
    (i64.const {k_uri_p}) (i64.const {k_uri_l})
    (i64.extend_i32_u (local.get $uri_p)) (local.get $uri_l) (i64.const 1)))
  (if (local.get $mb)
    (then (call $__rt_heap_free (local.get $mb))))                ;; hash_set copied the normalized mode
  (local.set $out (call $__rt_mixed_from_value (i64.const 5)
    (i64.extend_i32_u (local.get $hash)) (i64.const 0)))
  (call $__rt_decref_any (local.get $hash))                       ;; the cell is the hash's sole owner
  (local.get $out))
"#,
        flag = MEMSTREAM_FLAG,
        meta = FD_STREAM_META,
        php_p = v_php.0, php_l = v_php.1,
        temp_p = v_temp.0, temp_l = v_temp.1,
        mem_p = v_mem.0, mem_l = v_mem.1,
        plain_p = v_plain.0, plain_l = v_plain.1,
        stdio_p = v_stdio.0, stdio_l = v_stdio.1,
        k_to_p = k_to.0, k_to_l = k_to.1,
        k_bl_p = k_bl.0, k_bl_l = k_bl.1,
        k_eof_p = k_eof.0, k_eof_l = k_eof.1,
        k_wt_p = k_wt.0, k_wt_l = k_wt.1,
        k_st_p = k_st.0, k_st_l = k_st.1,
        k_mode_p = k_mode.0, k_mode_l = k_mode.1,
        k_ub_p = k_ub.0, k_ub_l = k_ub.1,
        k_seek_p = k_seek.0, k_seek_l = k_seek.1,
        k_uri_p = k_uri.0, k_uri_l = k_uri.1,
    ));
}

/// `__rt_is_data_uri`: whether a path is an RFC 2397 `data:` URI.
///
/// The `//` is OPTIONAL — measured on php-src 8.5.6, `data:,x` opens exactly like
/// `data://,x` — so only the five-byte `data:` prefix is tested here.
const RT_IS_DATA_URI: &str = r#"(func $__rt_is_data_uri (param $p i32) (param $len i64) (result i32)
  (if (i64.lt_u (local.get $len) (i64.const 5))
    (then (return (i32.const 0))))
  (i32.and
    (i32.and (i32.eq (i32.load8_u (local.get $p)) (i32.const 100))                     ;; 'd'
             (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 1))) (i32.const 97)))  ;; 'a'
    (i32.and
      (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 2))) (i32.const 116))   ;; 't'
               (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 3))) (i32.const 97)))   ;; 'a'
      (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.const 4))) (i32.const 58)))))          ;; ':'
"#;

/// `__rt_data_uri_open`: decodes an RFC 2397 `data:` URI into an in-memory stream.
///
/// Answers the memstream handle, or -1 when the URI cannot be opened — which php reports as
/// `fopen`'s `false`. Measured on php-src 8.5.6, every rule below came from a probe rather
/// than from the RFC:
///   * the `//` after `data:` is optional;
///   * NO comma at all fails (`data://text/plain` is false);
///   * the payload is base64 only when the media type ends with the LOWERCASE `;base64` —
///     `;BASE64` fails, because php compares that literal case-sensitively;
///   * base64 decoding is STRICT here, unlike the `base64_decode` builtin: whitespace and `=`
///     are skipped but any other stray byte fails the open (`;base64,####` is false);
///     padding is optional (`SGVsbG8` decodes like `SGVsbG8=`);
///   * a plain payload is `urldecode`, NOT `rawurldecode`: `+` becomes a space. A `%` without
///     two hex digits after it stays LITERAL (`100%` is `100%`, `%4` is `%4`, `%zz` is `%zz`);
///   * an empty payload is a valid empty stream.
const RT_DATA_URI_OPEN: &str = r#"(func $__rt_data_uri_open (param $p i32) (param $len i64) (param $mode i32) (param $mode_len i64) (result i32)
  (local $i i32) (local $n i32) (local $comma i32) (local $meta i32)
  (local $b64 i32) (local $buf i32) (local $w i32) (local $c i32)
  (local $v i32) (local $acc i32) (local $bits i32) (local $hi i32) (local $lo i32)
  (local $h i32) (local $mend i32) (local $seg i32)
  (local.set $n (i32.wrap_i64 (local.get $len)))
  (local.set $i (i32.const 5))                                    ;; past "data:"
  (if (i32.and (i32.gt_s (i32.sub (local.get $n) (local.get $i)) (i32.const 1))
       (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (local.get $i))) (i32.const 47))
                (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 1))))
                        (i32.const 47))))
    (then (local.set $i (i32.add (local.get $i) (i32.const 2)))))  ;; the optional "//"
  (local.set $meta (local.get $i))
  (local.set $comma (i32.const -1))
  (block $found (loop $scan                                        ;; the FIRST comma splits it
    (br_if $found (i32.ge_s (local.get $i) (local.get $n)))
    (if (i32.eq (i32.load8_u (i32.add (local.get $p) (local.get $i))) (i32.const 44))
      (then
        (local.set $comma (local.get $i))
        (br $found)))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $scan)))
  (if (i32.lt_s (local.get $comma) (i32.const 0))
    (then (return (i32.const -1))))                                ;; no comma: php answers false
  ;; base64 when the media type ends with the lowercase ";base64"
  (local.set $b64 (i32.const 0))
  (if (i32.ge_s (i32.sub (local.get $comma) (local.get $meta)) (i32.const 7))
    (then
      (local.set $i (i32.sub (local.get $comma) (i32.const 7)))
      (if (i32.and
            (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (local.get $i))) (i32.const 59))
                     (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 1)))) (i32.const 98)))
            (i32.and
              (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 2)))) (i32.const 97))
                       (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 3)))) (i32.const 115)))
              (i32.and
                (i32.and (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 4)))) (i32.const 101))
                         (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 5)))) (i32.const 54)))
                (i32.eq (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 6)))) (i32.const 52)))))
        (then (local.set $b64 (i32.const 1))))))
  ;; Every remaining `;` parameter must be `key=value`. Measured: `;charset=utf-8`,
  ;; `;CHARSET=utf-8` and `;foo=bar` all open, while `;foo`, `;base64x`, `;BASE64` and a bare
  ;; `;` all answer FALSE — so the lowercase `base64` above is the ONLY parameter allowed to
  ;; carry no `=`, and an uppercase spelling is not a base64 marker NOR a valid parameter.
  (local.set $mend (local.get $comma))
  (if (local.get $b64)
    (then (local.set $mend (i32.sub (local.get $comma) (i32.const 7)))))
  (local.set $i (local.get $meta))
  (local.set $seg (i32.const -1))                                  ;; -1 until the first ';'
  (block $mdone (loop $mscan
    (if (i32.ge_s (local.get $i) (local.get $mend))
      (then
        (if (i32.eq (local.get $seg) (i32.const 0))                ;; a parameter closed with no '='
          (then (return (i32.const -1))))                          ;; nothing allocated yet
        (br $mdone)))
    (local.set $c (i32.load8_u (i32.add (local.get $p) (local.get $i))))
    (if (i32.eq (local.get $c) (i32.const 59))                     ;; ';' closes a segment
      (then
        (if (i32.eq (local.get $seg) (i32.const 0))
          (then (return (i32.const -1))))
        (local.set $seg (i32.const 0)))                            ;; the next one needs an '='
      (else
        (if (i32.and (i32.eq (local.get $c) (i32.const 61))        ;; '=' satisfies it
                     (i32.eq (local.get $seg) (i32.const 0)))
          (then (local.set $seg (i32.const 1))))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $mscan)))
  (local.set $i (i32.add (local.get $comma) (i32.const 1)))        ;; the payload starts here
  ;; Neither decoding ever grows the input, so the payload length bounds both.
  (local.set $buf (call $__rt_heap_alloc (i32.add (i32.sub (local.get $n) (local.get $i)) (i32.const 1))))
  (local.set $w (i32.const 0))
  (if (local.get $b64)
    (then
      (block $bend (loop $b64loop
        (br_if $bend (i32.ge_s (local.get $i) (local.get $n)))
        (local.set $c (i32.load8_u (i32.add (local.get $p) (local.get $i))))
        (local.set $v (call $__rt_b64_value (local.get $c)))
        (if (i32.ge_s (local.get $v) (i32.const 0))
          (then
            (local.set $acc (i32.or (i32.shl (local.get $acc) (i32.const 6)) (local.get $v)))
            (local.set $bits (i32.add (local.get $bits) (i32.const 6)))
            (if (i32.ge_u (local.get $bits) (i32.const 8))
              (then
                (local.set $bits (i32.sub (local.get $bits) (i32.const 8)))
                (i32.store8 (i32.add (local.get $buf) (local.get $w))
                  (i32.and (i32.shr_u (local.get $acc) (local.get $bits)) (i32.const 255)))
                (local.set $w (i32.add (local.get $w) (i32.const 1))))))
          (else
            ;; STRICT: only padding and whitespace may sit outside the alphabet.
            (if (i32.eqz (i32.or
                  (i32.eq (local.get $c) (i32.const 61))                       ;; '='
                  (i32.or
                    (i32.eq (local.get $c) (i32.const 32))                     ;; ' '
                    (i32.and (i32.ge_u (local.get $c) (i32.const 9))
                             (i32.le_u (local.get $c) (i32.const 13))))))      ;; \t \n \v \f \r
              (then
                (call $__rt_heap_free (local.get $buf))
                (return (i32.const -1))))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $b64loop))))
    (else
      (block $pend (loop $pctloop
        (br_if $pend (i32.ge_s (local.get $i) (local.get $n)))
        (local.set $c (i32.load8_u (i32.add (local.get $p) (local.get $i))))
        (if (i32.eq (local.get $c) (i32.const 43))                             ;; '+' is a space
          (then
            (i32.store8 (i32.add (local.get $buf) (local.get $w)) (i32.const 32))
            (local.set $w (i32.add (local.get $w) (i32.const 1))))
          (else
            (local.set $hi (i32.const -1))
            (if (i32.and (i32.eq (local.get $c) (i32.const 37))                ;; '%'
                         (i32.lt_s (i32.add (local.get $i) (i32.const 2)) (local.get $n)))
              (then
                (local.set $hi (call $__rt_hex_digit_value
                  (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 1))))))
                (local.set $lo (call $__rt_hex_digit_value
                  (i32.load8_u (i32.add (local.get $p) (i32.add (local.get $i) (i32.const 2))))))
                (if (i32.lt_s (local.get $lo) (i32.const 0))
                  (then (local.set $hi (i32.const -1))))))
            (if (i32.ge_s (local.get $hi) (i32.const 0))
              (then                                                            ;; a complete %HH
                (i32.store8 (i32.add (local.get $buf) (local.get $w))
                  (i32.add (i32.mul (local.get $hi) (i32.const 16)) (local.get $lo)))
                (local.set $i (i32.add (local.get $i) (i32.const 2))))
              (else                                                            ;; anything else is literal
                (i32.store8 (i32.add (local.get $buf) (local.get $w)) (local.get $c))))
            (local.set $w (i32.add (local.get $w) (i32.const 1)))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $pctloop)))))
  (local.set $h (call $__rt_memstream_new (local.get $mode) (local.get $mode_len)
                                          (local.get $p) (local.get $len)))
  (drop (call $__rt_memstream_write (local.get $h) (local.get $buf) (i64.extend_i32_u (local.get $w))))
  (call $__rt_heap_free (local.get $buf))
  (call $__rt_memstream_seek (local.get $h) (i64.const 0))         ;; reads start at the beginning
  (local.get $h))
"#;

/// `__rt_stream_get_line`: PHP's `stream_get_line`, boxed as `string|false`.
///
/// Measured on php-src 8.5.6, the rules the two branches implement:
///   * the delimiter is STRIPPED from the answer and consumed from the stream;
///   * the length cap wins over the delimiter and consumes NOTHING past the cap —
///     `stream_get_line($f, 3, "#")` on `abc#rest` answers `abc` with `ftell` at 3;
///   * a length of 0 (PHP's default) reads up to the 8192-byte default chunk;
///   * partial delimiter matches are data (`ab##` with `###` answers `ab##` whole);
///   * EOF with no bytes answers `false`; either EOF sets the feof flag, but a read
///     that stops at the cap — even on the last byte — does NOT.
///
/// The real-fd branch reads one byte at a time: WASI has no ungetc, so a byte read
/// past the delimiter could not be handed back, and byte-wise reads make the
/// stop-at-delimiter position exact by construction.
fn rt_stream_get_line() -> String {
    format!(
        r#"(func $__rt_stream_get_line (param $h i32) (param $len i64) (param $dptr i32) (param $dlen i64) (result i32)
  (local $eff i32) (local $d i32) (local $size i64) (local $pos i64) (local $buf i32)
  (local $avail i64) (local $limit i32) (local $k i32) (local $j i32) (local $m i32)
  (local $hit i32) (local $acc i32) (local $n i32) (local $out i32) (local $eof i32)
  (if (i32.lt_s (local.get $h) (i32.const 0))
    (then (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $eff (i32.const 8192))                               ;; PHP's default chunk for length 0
  (if (i64.gt_s (local.get $len) (i64.const 0))
    (then (local.set $eff (i32.wrap_i64 (local.get $len)))))
  (if (i32.and (local.get $h) (i32.const {flag}))
    (then
      ;; In-memory stream: the bytes are already here, scan them in place.
      (local.set $d (i32.and (local.get $h) (i32.const 1073741823)))
      (local.set $size (i64.load (local.get $d)))
      (local.set $pos (i64.load (i32.add (local.get $d) (i32.const 16))))
      (local.set $avail (i64.sub (local.get $size) (local.get $pos)))
      (if (i64.le_s (local.get $avail) (i64.const 0))
        (then
          (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 1))  ;; the read found nothing
          (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
      (local.set $buf (i32.add (i32.load (i32.add (local.get $d) (i32.const 32)))
                               (i32.wrap_i64 (local.get $pos))))
      (local.set $limit (i32.wrap_i64 (local.get $avail)))
      (if (i32.gt_u (local.get $limit) (local.get $eff))
        (then (local.set $limit (local.get $eff))))
      (local.set $hit (i32.const -1))
      (if (i64.gt_s (local.get $dlen) (i64.const 0))
        (then
          (local.set $k (i32.const 0))
          (block $found (loop $scan
            (br_if $found (i32.ge_u (local.get $k) (local.get $limit)))
            ;; the delimiter must fit in the STREAM's remaining bytes, not the cap window
            (if (i64.le_s (i64.add (i64.extend_i32_u (local.get $k)) (local.get $dlen)) (local.get $avail))
              (then
                (local.set $j (i32.const 0))
                (local.set $m (i32.const 1))
                (block $cmp_done (loop $cmp
                  (br_if $cmp_done (i32.ge_u (local.get $j) (i32.wrap_i64 (local.get $dlen))))
                  (if (i32.ne
                        (i32.load8_u (i32.add (i32.add (local.get $buf) (local.get $k)) (local.get $j)))
                        (i32.load8_u (i32.add (local.get $dptr) (local.get $j))))
                    (then
                      (local.set $m (i32.const 0))
                      (br $cmp_done)))
                  (local.set $j (i32.add (local.get $j) (i32.const 1)))
                  (br $cmp)))
                (if (local.get $m)
                  (then
                    (local.set $hit (local.get $k))
                    (br $found)))))
            (local.set $k (i32.add (local.get $k) (i32.const 1)))
            (br $scan)))))
      (if (i32.ne (local.get $hit) (i32.const -1))
        (then
          (local.set $out (call $__rt_mixed_from_value (i64.const 1)
            (i64.extend_i32_u (local.get $buf)) (i64.extend_i32_u (local.get $hit))))
          (i64.store (i32.add (local.get $d) (i32.const 16))      ;; consume data AND delimiter
            (i64.add (local.get $pos)
              (i64.add (i64.extend_i32_u (local.get $hit)) (local.get $dlen))))
          (return (local.get $out))))
      (local.set $out (call $__rt_mixed_from_value (i64.const 1)
        (i64.extend_i32_u (local.get $buf)) (i64.extend_i32_u (local.get $limit))))
      (i64.store (i32.add (local.get $d) (i32.const 16))          ;; cap or end: consume only the data
        (i64.add (local.get $pos) (i64.extend_i32_u (local.get $limit))))
      ;; No delimiter and the DATA ran out before the cap: the fd branch would have
      ;; attempted one more read and found nothing, so this stop is an EOF stop. A stop
      ;; AT the cap never attempts that read — even when the cap lands on the last byte.
      (if (i32.lt_u (local.get $limit) (local.get $eff))
        (then (i64.store (i32.add (local.get $d) (i32.const 24)) (i64.const 1))))
      (return (local.get $out))))
  ;; Real fd: one byte at a time, stopping on cap, delimiter, or the read that finds nothing.
  (local.set $acc (call $__rt_heap_alloc (local.get $eff)))
  (block $stop (loop $byte
    (br_if $stop (i32.ge_u (local.get $n) (local.get $eff)))      ;; cap: eof stays untouched
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov}))
      (i32.add (local.get $acc) (local.get $n)))
    (i32.store (i32.add (global.get $__float_scratch) (i32.const {iov_len})) (i32.const 1))
    (if (i32.ne (call $wasi_fd_read
          (local.get $h)
          (i32.add (global.get $__float_scratch) (i32.const {iov}))
          (i32.const 1)
          (i32.add (global.get $__float_scratch) (i32.const {nread}))) (i32.const 0))
      (then
        (local.set $eof (i32.const 1))
        (br $stop)))
    (if (i32.eqz (i32.load (i32.add (global.get $__float_scratch) (i32.const {nread}))))
      (then
        (local.set $eof (i32.const 1))
        (br $stop)))
    (local.set $n (i32.add (local.get $n) (i32.const 1)))
    (if (i32.and (i64.gt_s (local.get $dlen) (i64.const 0))
                 (i32.ge_u (local.get $n) (i32.wrap_i64 (local.get $dlen))))
      (then
        ;; does the accumulator now END WITH the delimiter?
        (local.set $j (i32.const 0))
        (local.set $m (i32.const 1))
        (block $cmp_done (loop $cmp
          (br_if $cmp_done (i32.ge_u (local.get $j) (i32.wrap_i64 (local.get $dlen))))
          (if (i32.ne
                (i32.load8_u (i32.add
                  (i32.add (local.get $acc) (i32.sub (local.get $n) (i32.wrap_i64 (local.get $dlen))))
                  (local.get $j)))
                (i32.load8_u (i32.add (local.get $dptr) (local.get $j))))
            (then
              (local.set $m (i32.const 0))
              (br $cmp_done)))
          (local.set $j (i32.add (local.get $j) (i32.const 1)))
          (br $cmp)))
        (if (local.get $m)
          (then
            (local.set $out (call $__rt_mixed_from_value (i64.const 1)
              (i64.extend_i32_u (local.get $acc))
              (i64.extend_i32_u (i32.sub (local.get $n) (i32.wrap_i64 (local.get $dlen))))))
            (call $__rt_heap_free (local.get $acc))
            (return (local.get $out))))))
    (br $byte)))
  (if (local.get $eof)
    (then (call $__rt_fd_eof_set (local.get $h))))
  (if (i32.and (local.get $eof) (i32.eqz (local.get $n)))
    (then
      (call $__rt_heap_free (local.get $acc))
      (return (call $__rt_mixed_from_value (i64.const 3) (i64.const 0) (i64.const 0)))))
  (local.set $out (call $__rt_mixed_from_value (i64.const 1)
    (i64.extend_i32_u (local.get $acc)) (i64.extend_i32_u (local.get $n))))
  (call $__rt_heap_free (local.get $acc))
  (local.get $out))
"#,
        flag = MEMSTREAM_FLAG,
        iov = IO_SCRATCH,
        iov_len = IO_SCRATCH + 4,
        nread = IO_SCRATCH + 8
    )
}
