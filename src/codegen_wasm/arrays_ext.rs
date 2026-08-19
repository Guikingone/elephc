//! Purpose:
//! The array builtins that read or rebuild a whole array: `array_reverse` over any element
//! representation, `array_sum` over a heterogeneous one, `array_pad`, `array_unique` and
//! `array_column`.
//!
//! Called from:
//! - `builtins::emit_builtin_runtime` — `emit_array_ext_runtime`.
//! - `builtins` — the shape checks and the lowerings.
//!
//! Key details:
//! - Where an operation only MOVES elements, it is generic: `__rt_array_clone_shallow` already
//!   copies the payload bytes and increfs every refcounted child, so reversing is a swap of
//!   slots on the clone and the element representation is preserved untouched. Nothing here has
//!   to know whether a slot holds an int, a string pair or a Mixed cell.
//! - Where an operation has to WRITE a new element, it cannot be generic: the value's width and
//!   its refcount obligation both follow the array's `value_type`. Those are served for the raw
//!   i64 representation and refused otherwise, which is the same line `__rt_array_reverse_int`
//!   and `__rt_array_sum_int` already draw.
//! - `array_column` is the exception that is naturally heterogeneous: its rows are hashes and
//!   its result is `array<mixed>`, so every value is boxed and the result carries cells.

use super::wat::WatModule;

/// Adds the array-rebuilding runtime. References only helpers the array, hash and mixed
/// runtimes already emit.
pub(super) fn emit_array_ext_runtime(wm: &mut WatModule, has_main: bool) {
    wm.add_raw_func(RT_ARRAY_REVERSE_ANY);
    wm.add_raw_func(RT_ARRAY_PAD_INT);
    wm.add_raw_func(RT_ARRAY_UNIQUE_INT);
    wm.add_raw_func(RT_ARRAY_CLONE_RANGE);
    wm.add_raw_func(RT_ARRAY_SPLICE_ANY);
    wm.add_raw_func(RT_ARRAY_UNSHIFT_INT);
    // These two read their elements through the GENERIC walk, which belongs to the command
    // runtime because its `foreach` dispatch warns through WASI. A reactor module has neither.
    if !has_main {
        return;
    }
    wm.add_raw_func(RT_ARRAY_SUM_MIXED);
    wm.add_raw_func(RT_ARRAY_COLUMN);
}

/// `__rt_array_reverse_any`: `array_reverse` for any element representation.
///
/// The clone carries the source's `value_type`, `elem_size` and one reference per refcounted
/// child, so reversing is a byte swap of whole slots: nothing is boxed, nothing is re-tagged,
/// and the refcounts are already right because the elements are the same objects.
const RT_ARRAY_REVERSE_ANY: &str = r#"(func $__rt_array_reverse_any (param $array i32) (result i32)
  (local $out i32) (local $len i64) (local $esz i32) (local $i i64) (local $j i64)
  (local $a i32) (local $b i32) (local $k i32) (local $tmp i32)
  (local.set $out (call $__rt_array_clone_shallow (local.get $array)))
  (local.set $len (i64.load (local.get $out)))
  (local.set $esz (i32.wrap_i64 (i64.load offset=16 (local.get $out))))
  (local.set $i (i64.const 0))
  (local.set $j (i64.sub (local.get $len) (i64.const 1)))
  (block $done (loop $swap
    (br_if $done (i64.ge_s (local.get $i) (local.get $j)))
    (local.set $a (i32.add (i32.add (local.get $out) (i32.const 24))
                           (i32.mul (i32.wrap_i64 (local.get $i)) (local.get $esz))))
    (local.set $b (i32.add (i32.add (local.get $out) (i32.const 24))
                           (i32.mul (i32.wrap_i64 (local.get $j)) (local.get $esz))))
    (local.set $k (i32.const 0))
    (block $swapped (loop $byte
      (br_if $swapped (i32.ge_u (local.get $k) (local.get $esz)))
      (local.set $tmp (i32.load8_u (i32.add (local.get $a) (local.get $k))))
      (i32.store8 (i32.add (local.get $a) (local.get $k))
                  (i32.load8_u (i32.add (local.get $b) (local.get $k))))
      (i32.store8 (i32.add (local.get $b) (local.get $k)) (local.get $tmp))
      (local.set $k (i32.add (local.get $k) (i32.const 1)))
      (br $byte)))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (local.set $j (i64.sub (local.get $j) (i64.const 1)))
    (br $swap)))
  (local.get $out))
"#;

/// `__rt_array_sum_mixed`: `array_sum` over a heterogeneous array.
///
/// Each element is read through the generic walk and cast with php's own integer conversion, so
/// a numeric string sums as its number exactly as php does. The result is an integer because the
/// checker proved every element is one; a float element would have made the call's result type a
/// float, which the audit refuses.
const RT_ARRAY_SUM_MIXED: &str = r#"(func $__rt_array_sum_mixed (param $array i32) (result i64)
  (local $acc i64) (local $cursor i64) (local $more i64) (local $next i64) (local $cell i32)
  (local.set $cursor (i64.const -1))
  (block $done (loop $entry
    (call $__rt_mixed_iter_next (local.get $array) (local.get $cursor) (i32.const 0))
    (local.set $more)
    (local.set $next)
    (br_if $done (i64.eqz (local.get $more)))
    (local.set $cursor (local.get $next))
    (local.set $cell (call $__rt_mixed_iter_value (local.get $array) (local.get $cursor) (i32.const 0)))
    (local.set $acc (i64.add (local.get $acc) (call $__rt_mixed_cast_int (local.get $cell))))
    (call $__rt_decref_any (local.get $cell))
    (br $entry)))
  (local.get $acc))
"#;

/// `__rt_array_pad_int`: `array_pad` over raw i64 slots.
///
/// php pads on the RIGHT for a positive size and on the LEFT for a negative one, and answers a
/// copy untouched when the array is already that long — the size is a target, not an amount.
const RT_ARRAY_PAD_INT: &str = r#"(func $__rt_array_pad_int (param $array i32) (param $size i64) (param $value i64) (result i32)
  (local $len i64) (local $want i64) (local $out i32) (local $i i64)
  (local.set $len (i64.load (local.get $array)))
  (local.set $want (select (i64.sub (i64.const 0) (local.get $size)) (local.get $size)
                           (i64.lt_s (local.get $size) (i64.const 0))))
  (if (i64.le_s (local.get $want) (local.get $len))
    (then (return (call $__rt_array_clone_shallow (local.get $array)))))
  (local.set $out (call $__rt_array_new (local.get $want) (i64.const 8)))
  (if (i64.lt_s (local.get $size) (i64.const 0))
    (then
      ;; A negative size pads on the left, so the fill comes first.
      (local.set $i (local.get $len))
      (block $filled (loop $pad
        (br_if $filled (i64.ge_s (local.get $i) (local.get $want)))
        (local.set $out (call $__rt_array_push_int (local.get $out) (local.get $value)))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $pad)))))
  (local.set $i (i64.const 0))
  (block $copied (loop $element
    (br_if $copied (i64.ge_s (local.get $i) (local.get $len)))
    (local.set $out (call $__rt_array_push_int (local.get $out)
      (i64.load (i32.add (i32.add (local.get $array) (i32.const 24))
                         (i32.wrap_i64 (i64.mul (local.get $i) (i64.const 8)))))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $element)))
  (if (i64.ge_s (local.get $size) (i64.const 0))
    (then
      (local.set $i (local.get $len))
      (block $filled (loop $pad
        (br_if $filled (i64.ge_s (local.get $i) (local.get $want)))
        (local.set $out (call $__rt_array_push_int (local.get $out) (local.get $value)))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $pad)))))
  (local.get $out))
"#;

/// `__rt_array_unique_int`: `array_unique` over raw i64 slots.
///
/// php keeps the FIRST occurrence of each value and preserves the original keys, so the result
/// of a list is a list with holes. This target has no sparse indexed representation, so the
/// result is re-indexed — a divergence the audit confines to a source whose keys the program
/// never reads back, which is why `array_unique` is only admitted where its result feeds a
/// `foreach` over values.
const RT_ARRAY_UNIQUE_INT: &str = r#"(func $__rt_array_unique_int (param $array i32) (result i32)
  (local $len i64) (local $out i32) (local $i i64) (local $j i64) (local $value i64) (local $seen i32)
  (local.set $len (i64.load (local.get $array)))
  (local.set $out (call $__rt_array_new (local.get $len) (i64.const 8)))
  (local.set $i (i64.const 0))
  (block $done (loop $element
    (br_if $done (i64.ge_s (local.get $i) (local.get $len)))
    (local.set $value (i64.load (i32.add (i32.add (local.get $array) (i32.const 24))
                                         (i32.wrap_i64 (i64.mul (local.get $i) (i64.const 8))))))
    (local.set $seen (i32.const 0))
    (local.set $j (i64.const 0))
    (block $scanned (loop $earlier
      (br_if $scanned (i64.ge_s (local.get $j) (local.get $i)))
      (if (i64.eq (local.get $value)
                  (i64.load (i32.add (i32.add (local.get $array) (i32.const 24))
                                     (i32.wrap_i64 (i64.mul (local.get $j) (i64.const 8))))))
        (then (local.set $seen (i32.const 1)) (br $scanned)))
      (local.set $j (i64.add (local.get $j) (i64.const 1)))
      (br $earlier)))
    (if (i32.eqz (local.get $seen))
      (then (local.set $out (call $__rt_array_push_int (local.get $out) (local.get $value)))))
    (local.set $i (i64.add (local.get $i) (i64.const 1)))
    (br $element)))
  (local.get $out))
"#;

/// `__rt_array_column`: the named value of every row, as an `array<mixed>`.
///
/// The rows are hashes, so each lookup goes through the hash getter and the found `(lo, hi, tag)`
/// triple is boxed — the result is heterogeneous by construction, which is what the checker types
/// it. A row missing the key contributes nothing, which is php's rule.
const RT_ARRAY_COLUMN: &str = r#"(func $__rt_array_column (param $array i32) (param $key i32) (param $key_len i64) (result i32)
  (local $out i32) (local $cursor i64) (local $more i64) (local $next i64)
  (local $row i32) (local $found i32) (local $lo i64) (local $hi i64) (local $tag i64)
  (local $cell i32)
  (local.set $out (call $__rt_array_new (i64.load (local.get $array)) (i64.const 16)))
  (local.set $cursor (i64.const -1))
  (block $done (loop $entry
    (call $__rt_mixed_iter_next (local.get $array) (local.get $cursor) (i32.const 0))
    (local.set $more)
    (local.set $next)
    (br_if $done (i64.eqz (local.get $more)))
    (local.set $cursor (local.get $next))
    (local.set $cell (call $__rt_mixed_iter_value (local.get $array) (local.get $cursor) (i32.const 0)))
    (call $__rt_mixed_unbox (local.get $cell))
    (local.set $hi)
    (local.set $lo)
    (local.set $tag)
    (local.set $row (i32.wrap_i64 (local.get $lo)))
    (call $__rt_decref_any (local.get $cell))
    (if (i64.ne (local.get $tag) (i64.const 5))                    ;; only a hash row has columns
      (then (br $entry)))
    (call $__rt_hash_get (local.get $row)
      (i64.extend_i32_u (local.get $key)) (local.get $key_len))
    (local.set $tag)
    (local.set $hi)
    (local.set $lo)
    (local.set $found)
    (if (i32.eqz (local.get $found))
      (then (br $entry)))
    (local.set $cell (call $__rt_mixed_from_value (local.get $tag) (local.get $lo) (local.get $hi)))
    (local.set $out (call $__rt_array_push_mixed (local.get $out) (local.get $cell)))
    (br $entry)))
  (local.get $out))
"#;

/// `__rt_array_clone_range`: an independent array holding `count` elements from `start`.
///
/// The ownership fixup is `__rt_array_clone_shallow`'s, bounded to the range: a string element
/// is persisted into its own copy and a refcounted container child is increfed, because the copy
/// is a second owner. Anything else is bytes, which the copy already carries.
const RT_ARRAY_CLONE_RANGE: &str = r#"(func $__rt_array_clone_range (param $array i32) (param $start i64) (param $count i64) (result i32)
  (local $esz i64) (local $kindw i64) (local $new i32) (local $vt i32) (local $i i64)
  (local $slot i32) (local $oldptr i32) (local $slen i64) (local $np i32) (local $nl i64)
  (local $src i32) (local $dst i32) (local $j i64)
  (local.set $esz (i64.load offset=16 (local.get $array)))
  (local.set $kindw (i64.load (i32.sub (local.get $array) (i32.const 8))))
  (if (i64.lt_s (local.get $count) (i64.const 0))
    (then (local.set $count (i64.const 0))))
  (local.set $new (call $__rt_array_new
    (select (local.get $count) (i64.const 1) (i64.gt_s (local.get $count) (i64.const 0)))
    (local.get $esz)))
  (i64.store (i32.sub (local.get $new) (i32.const 8))
             (i64.and (local.get $kindw) (i64.const 65535)))       ;; same kind/value_type/COW
  (i64.store (local.get $new) (local.get $count))
  (local.set $src (i32.add (i32.add (local.get $array) (i32.const 24))
                           (i32.wrap_i64 (i64.mul (local.get $start) (local.get $esz)))))
  (local.set $dst (i32.add (local.get $new) (i32.const 24)))
  (local.set $j (i64.mul (local.get $count) (local.get $esz)))
  (block $bend (loop $bcopy
    (br_if $bend (i64.le_s (local.get $j) (i64.const 0)))
    (i32.store8 (local.get $dst) (i32.load8_u (local.get $src)))
    (local.set $src (i32.add (local.get $src) (i32.const 1)))
    (local.set $dst (i32.add (local.get $dst) (i32.const 1)))
    (local.set $j (i64.sub (local.get $j) (i64.const 1)))
    (br $bcopy)))
  (local.set $vt (i32.and (i32.wrap_i64 (i64.shr_u (local.get $kindw) (i64.const 8))) (i32.const 127)))
  (if (i32.eq (local.get $vt) (i32.const 1))                       ;; string elements own their bytes
    (then
      (local.set $i (i64.const 0))
      (block $send (loop $sclone
        (br_if $send (i64.ge_s (local.get $i) (local.get $count)))
        (local.set $slot (i32.add (i32.add (local.get $new) (i32.const 24))
                                  (i32.wrap_i64 (i64.mul (local.get $i) (i64.const 16)))))
        (local.set $oldptr (i32.wrap_i64 (i64.load (local.get $slot))))
        (local.set $slen (i64.load offset=8 (local.get $slot)))
        (call $__rt_str_persist (local.get $oldptr) (local.get $slen))
        (local.set $nl)
        (local.set $np)
        (i64.store (local.get $slot) (i64.extend_i32_u (local.get $np)))
        (i64.store offset=8 (local.get $slot) (local.get $nl))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $sclone))))
    (else
      (if (i32.or (i32.eq (local.get $vt) (i32.const 4))
          (i32.or (i32.eq (local.get $vt) (i32.const 5))
          (i32.or (i32.eq (local.get $vt) (i32.const 6))
          (i32.or (i32.eq (local.get $vt) (i32.const 7))
                  (i32.eq (local.get $vt) (i32.const 10))))))      ;; refcounted children
        (then
          (local.set $i (i64.const 0))
          (block $rend (loop $rclone
            (br_if $rend (i64.ge_s (local.get $i) (local.get $count)))
            (call $__rt_incref (i32.wrap_i64 (i64.load
              (i32.add (i32.add (local.get $new) (i32.const 24))
                       (i32.wrap_i64 (i64.mul (local.get $i) (local.get $esz)))))))
            (local.set $i (i64.add (local.get $i) (i64.const 1)))
            (br $rclone)))))))
  (local.get $new))
"#;

/// `__rt_array_splice_any`: removes a window and inserts a replacement, answering both halves.
///
/// Returns `(out, removed)`. The caller writes `out` back through the array's storage and owns
/// `removed`; the SOURCE keeps its own reference, which the write-back then drops.
///
/// php's window normalisation is the fiddly part and is measured, not guessed: a negative offset
/// counts from the end, a negative length leaves that many elements at the end untouched, and an
/// offset past the end appends.
const RT_ARRAY_SPLICE_ANY: &str = r#"(func $__rt_array_splice_any (param $array i32) (param $off i64) (param $len i64) (param $has_len i32) (param $repl i32) (result i32 i32)
  (local $n i64) (local $start i64) (local $count i64) (local $out i32) (local $removed i32)
  (local.set $n (i64.load (local.get $array)))
  (local.set $start (local.get $off))
  (if (i64.lt_s (local.get $start) (i64.const 0))
    (then
      (local.set $start (i64.add (local.get $n) (local.get $start)))
      (if (i64.lt_s (local.get $start) (i64.const 0))
        (then (local.set $start (i64.const 0))))))
  (if (i64.gt_s (local.get $start) (local.get $n))
    (then (local.set $start (local.get $n))))
  (if (i32.eqz (local.get $has_len))
    (then (local.set $count (i64.sub (local.get $n) (local.get $start))))
    (else
      (local.set $count (local.get $len))
      (if (i64.lt_s (local.get $count) (i64.const 0))
        (then
          (local.set $count (i64.add (i64.sub (local.get $n) (local.get $start)) (local.get $count)))
          (if (i64.lt_s (local.get $count) (i64.const 0))
            (then (local.set $count (i64.const 0))))))))
  (if (i64.gt_s (i64.add (local.get $start) (local.get $count)) (local.get $n))
    (then (local.set $count (i64.sub (local.get $n) (local.get $start)))))
  (local.set $removed (call $__rt_array_clone_range (local.get $array) (local.get $start) (local.get $count)))
  (local.set $out (call $__rt_array_clone_range (local.get $array) (i64.const 0) (local.get $start)))
  (if (local.get $repl)
    (then (local.set $out (call $__rt_array_append_from (local.get $out) (local.get $repl) (i64.const 0)))))
  (local.set $out (call $__rt_array_append_from (local.get $out) (local.get $array)
    (i64.add (local.get $start) (local.get $count))))
  (local.get $out)
  (local.get $removed))
"#;

/// `__rt_array_unshift_int`: prepends a list of raw i64 values, answering the new count.
///
/// Only the raw i64 representation, for the reason `array_pad` is: the prepended elements are
/// WRITTEN, and a value's width and refcount obligation both follow the array's `value_type`.
const RT_ARRAY_UNSHIFT_INT: &str = r#"(func $__rt_array_unshift_int (param $array i32) (param $values i32) (result i32 i64)
  (local $out i32)
  (local.set $out (call $__rt_array_clone_range (local.get $values) (i64.const 0)
                                                (i64.load (local.get $values))))
  (local.set $out (call $__rt_array_append_from (local.get $out) (local.get $array) (i64.const 0)))
  (local.get $out)
  (i64.load (local.get $out)))
"#;
