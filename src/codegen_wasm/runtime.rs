//! Purpose:
//! Emits the hand-authored WebAssembly text (WAT) runtime for the wasm32-wasi
//! backend: the WASI imports a command module needs and the `__rt_*` helper
//! routines (currently integer echo). Runtime helpers are added to a `WatModule`
//! as raw `(func ...)` blocks.
//!
//! Called from:
//! - `crate::codegen_wasm::generate()` for command (main-bearing) modules.
//!
//! Key details:
//! - Low linear memory is reserved as runtime scratch:
//!     [0, 8)        iovec for `fd_write`: { buf_ptr @0 (i32), buf_len @4 (i32) }
//!     [8, 16)       `nwritten` cell for `fd_write` / `args_sizes_get` scratch (i32)
//!     [16, 64)      number-formatting buffer (itoa/ftoa), written back-to-front
//!     [64, 65600)   legacy concat reservation, retained for stable static-data offsets
//!   Compile-time data segments and the heap start at `RT_SCRATCH_END`.
//! - `__rt_concat` is heap-backed and bounds-checked. The legacy
//!   `$__concat_off` cursor remains as an ABI-compatible no-op for existing
//!   `ConcatReset` lowering; live strings never occupy the shared reservation.
//!   WASI imports and the echo/exit helpers are "command" runtime, emitted only
//!   for main-bearing modules (importing WASI forces `_start`-command semantics).

use super::wat::{DataSegment, FuncImport, Global, ValType, WatModule};

/// Base offset of the legacy string-concatenation reservation.
const CONCAT_BASE: u32 = 64;
/// Size of the legacy reservation retained to keep static-data addresses stable.
const CONCAT_SIZE: u32 = 65536;

/// First linear-memory offset available to data segments / the heap; everything
/// below this is reserved runtime scratch (number buffer + concat buffer).
pub(super) const RT_SCRATCH_END: u32 = CONCAT_BASE + CONCAT_SIZE;

/// Base of the dedicated float<->string scratch region. The strtod bignum buffers
/// (`__rt_digits_to_f64` / `__rt_str_to_f64`) and the later ftoa/itoa scratch live
/// here, above the concat buffer, so a parse or format never collides with an
/// in-flight string concatenation whose cursor would otherwise run through 0x4000.
/// Callers reach this base via the immutable `$__float_scratch` global.
pub(super) const FLOAT_SCRATCH_BASE: u32 = RT_SCRATCH_END;

/// Size of the float<->string scratch region. The strtod path uses offsets
/// 0..0x1200 (four 96-limb bignums at +0/+1024/+2048/+3072 and the digit buffer at
/// +4096); the ftoa/itoa scratch lands at +0x2000..+0x3000. 16 KiB bounds both.
pub(super) const FLOAT_SCRATCH_SIZE: u32 = 0x4000;

/// First byte reserved for command-runtime fatal diagnostics.
const COMMAND_DATA_BASE: u32 = FLOAT_SCRATCH_BASE + FLOAT_SCRATCH_SIZE;
const ERR_DIV_ZERO: &[u8] =
    b"PHP Fatal error: Uncaught DivisionByZeroError: Division by zero\n";
const ERR_MOD_ZERO: &[u8] =
    b"PHP Fatal error: Uncaught DivisionByZeroError: Modulo by zero\n";
const ERR_NEG_SHIFT: &[u8] =
    b"PHP Fatal error: Uncaught ArithmeticError: Bit shift by negative number\n";
const ERR_INTDIV_OVERFLOW: &[u8] = b"PHP Fatal error: Uncaught ArithmeticError: Division of PHP_INT_MIN by -1 is not an integer\n";
const ERR_WASI: &[u8] = b"PHP Fatal error: WASI operation failed\n";
const ERR_OOM: &[u8] = b"PHP Fatal error: Allowed memory size exhausted\n";
const ERR_HASH_APPEND_OCCUPIED: &[u8] =
    b"PHP Fatal error: Uncaught Error: Cannot add element to the array as the next element is already occupied\n";
const ERR_CALLABLE_DISPATCH: &[u8] =
    b"PHP Fatal error: Uncaught Error: Invalid callable dispatch\n";
const ERR_MIXED_HEAP_TYPE: &[u8] =
    b"PHP Fatal error: Uncaught TypeError: Value does not match the required heap type\n";
/// A PHP exception that reaches the top of `main` with no `catch` to receive it.
///
/// KNOWN DIVERGENCE: reference PHP names the class and message and prints the file, line and
/// stack trace (`Uncaught Exception: boom in /path.php:4`). Reproducing that needs the built-in
/// Throwable accessors, which this target does not lower yet, so the diagnostic is currently
/// class-agnostic. The EXIT STATUS is PHP's — 255 — which is the observable most callers act on.
const ERR_UNCAUGHT_EXCEPTION: &[u8] = b"PHP Fatal error: Uncaught exception\n";
const ERR_METHOD_CALL_PREFIX: &[u8] =
    b"PHP Fatal error: Uncaught Error: Call to a member function ";
const ERR_METHOD_CALL_SUFFIX: &[u8] = b"() on ";
const PHP_TYPE_INT: &[u8] = b"int\n";
const PHP_TYPE_STRING: &[u8] = b"string\n";
const PHP_TYPE_FLOAT: &[u8] = b"float\n";
const PHP_TYPE_BOOL: &[u8] = b"bool\n";
const PHP_TYPE_ARRAY: &[u8] = b"array\n";
const PHP_TYPE_NULL: &[u8] = b"null\n";
const PHP_TYPE_RESOURCE: &[u8] = b"resource\n";
const PHP_TYPE_CALLABLE: &[u8] = b"callable\n";
const PHP_TYPE_UNKNOWN: &[u8] = b"unknown\n";
const ERR_UNDEFINED_METHOD_PREFIX: &[u8] =
    b"PHP Fatal error: Uncaught Error: Call to undefined method ";
const ERR_UNDEFINED_METHOD_SEPARATOR: &[u8] = b"::";
const ERR_UNDEFINED_METHOD_SUFFIX: &[u8] = b"()\n";
/// A dynamic dispatch selecting a class php-src would not enter for lack of arguments. The class
/// and method names come from the runtime table and the call site, the two counts are rendered by
/// `__rt_itoa`, and the wording between them is php-src's own: `exactly` when every declared
/// parameter is required, `at least` when a default makes the counts differ. Measured on 8.5.6 —
/// a variadic tail does NOT soften the word.
///
/// KNOWN DIVERGENCE: php-src continues `, 0 passed in /path.php on line 9 and …` and closes with
/// a stack trace. This target reports no location tail, the same convention its internal-function
/// `TypeError`s already follow; the class, the message and the 255 exit status are PHP's.
const ERR_TOO_FEW_ARGS_PREFIX: &[u8] =
    b"PHP Fatal error: Uncaught ArgumentCountError: Too few arguments to function ";
const ERR_TOO_FEW_ARGS_PASSED: &[u8] = b"(), ";
const ERR_TOO_FEW_ARGS_EXACTLY: &[u8] = b" passed and exactly ";
const ERR_TOO_FEW_ARGS_AT_LEAST: &[u8] = b" passed and at least ";
const ERR_TOO_FEW_ARGS_SUFFIX: &[u8] = b" expected\n";
const WARN_UNDEFINED_ARRAY_KEY_PREFIX: &[u8] = b"Warning: Undefined array key ";
const WARN_QUOTE: &[u8] = b"\"";
const WARN_SUFFIX: &[u8] = b"\n";
/// PHP 8.5 alone diagnoses a float whose value no integer can represent. The
/// rendered float sits between the two fragments, formatted by `__rt_ftoa`.
const WARN_FLOAT_NOT_REPRESENTABLE_PREFIX: &[u8] = b"Warning: The float ";
const WARN_FLOAT_NOT_REPRESENTABLE_SUFFIX: &[u8] =
    b" is not representable as an int, cast occurred\n";
/// Arithmetic on a string carrying only a numeric prefix warns and uses the prefix.
const WARN_NON_NUMERIC_VALUE: &[u8] = b"Warning: A non-numeric value encountered\n";
/// The ONE diagnostic in PHP's whole scalar-cast family: `(string)` of an array. Measured —
/// `(int)`, `(float)` and `(bool)` of an array are all silent.
const WARN_ARRAY_TO_STRING: &[u8] = b"Warning: Array to string conversion\n";
/// PHP reports an object reaching a numeric cast, then uses 1. The class name sits between
/// the prefix and the per-target suffix.
const WARN_OBJECT_TO_SCALAR_PREFIX: &[u8] = b"Warning: Object of class ";
const WARN_OBJECT_TO_INT_SUFFIX: &[u8] = b" could not be converted to int\n";
const WARN_OBJECT_TO_FLOAT_SUFFIX: &[u8] = b" could not be converted to float\n";
/// `(string)` of an object without `__toString` is a FATAL, not a warning — the one place
/// this family stops at a diagnostic and terminates.
const ERR_OBJECT_TO_STRING_PREFIX: &[u8] = b"PHP Fatal error: Uncaught Error: Object of class ";
const ERR_OBJECT_TO_STRING_SUFFIX: &[u8] = b" could not be converted to string\n";
/// Arithmetic on a wholly non-numeric string is a PHP `TypeError`. Reported as an
/// uncaught fatal until this target gains exception support.
const ERR_UNSUPPORTED_OPERAND: &[u8] =
    b"PHP Fatal error: Uncaught TypeError: Unsupported operand types\n";
/// `str_repeat()` with a negative count is a PHP `ValueError`, catchable like any other.
const ERR_STR_REPEAT_NEGATIVE: &[u8] = b"PHP Fatal error: Uncaught ValueError: str_repeat(): Argument #2 ($times) must be greater than or equal to 0\n";
/// `str_pad()` with an empty pad string is a PHP `ValueError` when padding is actually needed.
const ERR_STR_PAD_EMPTY: &[u8] = b"PHP Fatal error: Uncaught ValueError: str_pad(): Argument #3 ($pad_string) must not be empty\n";
/// `explode()` with an empty separator is a PHP `ValueError`; without it the split loops.
const ERR_EXPLODE_EMPTY_SEP: &[u8] = b"PHP Fatal error: Uncaught ValueError: explode(): Argument #1 ($separator) must not be empty\n";
/// `str_split()` with a non-positive chunk length is a PHP `ValueError`.
const ERR_STR_SPLIT_LENGTH: &[u8] = b"PHP Fatal error: Uncaught ValueError: str_split(): Argument #2 ($length) must be greater than 0\n";
/// `chr()` outside `[0, 255]` still answers, wrapping modulo 256, but is deprecated since 8.5.
const DEPRECATED_CHR_RANGE: &[u8] = b"Deprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256\n";
/// `ord()` on anything but exactly one byte still answers, but is deprecated since 8.5.
const DEPRECATED_ORD_LENGTH: &[u8] =
    b"Deprecated: ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead\n";
/// PHP's IMPLICIT coercion at a declared `int` return, when the value still converts but
/// loses its fraction on the way. Measured on 8.5.6: a float and a float-shaped string get
/// different wordings, and both render the offending value between the fragments.
const DEPRECATED_FLOAT_TO_INT_PREFIX: &[u8] = b"Deprecated: Implicit conversion from float ";
const DEPRECATED_FLOAT_STR_TO_INT_PREFIX: &[u8] =
    b"Deprecated: Implicit conversion from float-string \"";
const DEPRECATED_TO_INT_SUFFIX: &[u8] = b" to int loses precision\n";
const DEPRECATED_STR_TO_INT_SUFFIX: &[u8] = b"\" to int loses precision\n";
/// A value no `int` can hold, returned from a function declared `int`, is a `TypeError`
/// naming the FUNCTION — `f()` for a plain function, `C::m()` for a method, which is
/// already how the EIR names both — and the type word that arrived.
const ERR_RETURN_TYPE_PREFIX: &[u8] = b"PHP Fatal error: Uncaught TypeError: ";
const ERR_RETURN_TYPE_MIDDLE: &[u8] = b"(): Return value must be of type ";
const ERR_RETURN_TYPE_SEPARATOR: &[u8] = b", ";
const ERR_RETURN_TYPE_SUFFIX: &[u8] = b" returned\n";
/// The same `TypeError` at an internal function's declared PARAMETER, which php-src words
/// differently and positions by argument number: `strtoupper(): Argument #1 ($string) must be of
/// type string, array given`. The function and parameter names travel from the call site, and the
/// word after the comma comes from `__rt_type_word_for_tag`, exactly as the return one does.
const ERR_ARGUMENT_TYPE_MIDDLE: &[u8] = b"(): Argument #";
const ERR_ARGUMENT_NAME_PREFIX: &[u8] = b" ($";
const ERR_ARGUMENT_TYPE_MUST_BE: &[u8] = b") must be of type ";
const ERR_ARGUMENT_TYPE_SUFFIX: &[u8] = b" given\n";
/// `null` at that same boundary is NOT a `TypeError`: measured on php-src 8.5.6, it still
/// converts — to `""` for a `string` parameter — after this deprecation. Only a value with no
/// conversion at all (array, object, resource) raises.
const DEPRECATED_ARGUMENT_NULL_PREFIX: &[u8] = b"Deprecated: ";
const DEPRECATED_ARGUMENT_NULL_MIDDLE: &[u8] = b"(): Passing null to parameter #";
const DEPRECATED_ARGUMENT_NULL_OF_TYPE: &[u8] = b") of type ";
const DEPRECATED_ARGUMENT_NULL_SUFFIX: &[u8] = b" is deprecated\n";
/// `foreach` over something that is neither an array nor an object. Measured on php-src 8.5.6:
/// it WARNS, names the type that arrived, and runs the body zero times — it does not raise.
const WARN_FOREACH_NON_ITERABLE_PREFIX: &[u8] =
    b"Warning: foreach() argument must be of type array|object, ";
const WARN_FOREACH_NON_ITERABLE_SUFFIX: &[u8] = b" given\n";
/// Arithmetic on an operand with no numeric meaning. php-src names BOTH operand types and the
/// operator — `Unsupported operand types: string % int` — where this target's older fixed
/// message named neither. The left word comes from `__rt_type_word_for_tag`; the operator and
/// the right word travel from the call site, which knows them statically.
const ERR_OPERAND_TYPES_PREFIX: &[u8] =
    b"PHP Fatal error: Uncaught TypeError: Unsupported operand types: ";
const ERR_OPERAND_TYPES_SPACE: &[u8] = b" ";
/// PHP names a closure by its CLASS in that message — measured: `Closure returned`, not
/// `callable returned`. Every first-class closure PHP builds is a `Closure`, so the word is
/// fixed even though this target keeps a callable as a descriptor rather than an object.
const PHP_CLASS_CLOSURE: &[u8] = b"Closure";
/// `count()` names the VALUE for a boolean — `true given`, not `bool given` — which is the one
/// place its word table parts ways with the declared-return one. Measured on php-src 8.5.6.
const PHP_VALUE_TRUE: &[u8] = b"true";
const PHP_VALUE_FALSE: &[u8] = b"false";
/// The message carries no tail at all: an internal function's `TypeError` stops at the word.
const ERR_COUNT_PREFIX: &[u8] =
    b"PHP Fatal error: Uncaught TypeError: count(): Argument #1 ($value) must be of type Countable|array, ";
const ERR_COUNT_SUFFIX: &[u8] = b" given\n";
/// `$s[$i]` outside the string answers the EMPTY string after this warning, which names the
/// index AS WRITTEN — a negative one is reported negative, not resolved from the end first.
/// Exact php-src 8.5 diagnostic for a property read through a null receiver. The property name
/// is written between the two fragments, so a live data segment is not needed for the message.
const WARN_PROPERTY_ON_NULL_PREFIX: &[u8] = b"Warning: Attempt to read property \"";
const WARN_PROPERTY_ON_NULL_SUFFIX: &[u8] = b"\" on null\n";
/// Indexing a NON-container scalar warns and answers null. PHP 8.3 renamed the value in this
/// message the same way it did for null, and for a boolean it names the VALUE rather than the
/// type — measured on 8.5.6: `on true`, not `on bool`. Each case is one complete message rather
/// than a prefix plus a word, because the two profiles do not agree on how many pieces there are.
const WARN_OFFSET_ON_TRUE: &[u8] = b"Warning: Trying to access array offset on true\n";
const WARN_OFFSET_ON_FALSE: &[u8] = b"Warning: Trying to access array offset on false\n";
const WARN_OFFSET_ON_INT: &[u8] = b"Warning: Trying to access array offset on int\n";
const WARN_OFFSET_ON_FLOAT: &[u8] = b"Warning: Trying to access array offset on float\n";
const WARN_OFFSET_ON_TYPE_BOOL: &[u8] =
    b"Warning: Trying to access array offset on value of type bool\n";
const WARN_OFFSET_ON_TYPE_INT: &[u8] =
    b"Warning: Trying to access array offset on value of type int\n";
const WARN_OFFSET_ON_TYPE_FLOAT: &[u8] =
    b"Warning: Trying to access array offset on value of type float\n";
/// A path that cannot be opened warns before answering `false`. php-src names the path and the
/// errno — `fopen(nope.txt): Failed to open stream: No such file or directory` — which needs a
/// strerror table this backend has no data for; these are the NATIVE backend's own wording, so
/// the two Elephc targets agree and both stop short of php-src's detail.
const WARN_FOPEN_FAILED: &[u8] = b"Warning: fopen(): Failed to open stream\n";
const WARN_FILE_GET_CONTENTS_FAILED: &[u8] =
    b"Warning: file_get_contents(): Failed to open stream\n";
const WARN_UNINIT_STRING_OFFSET: &[u8] = b"Warning: Uninitialized string offset ";
/// The newline closing that warning, in this emitter's own data group.
const WARN_OFFSET_NEWLINE: &[u8] = b"\n";
/// Reaching `string` or `bool` from a NaN still converts, but PHP 8.5 WARNS first — measured
/// raw, since an error handler hides the level. There is no notice on the way to `float`, where
/// NaN is an ordinary value.
const WARN_NAN_TO_STRING: &[u8] = b"Warning: unexpected NAN value was coerced to string\n";
const WARN_NAN_TO_BOOL: &[u8] = b"Warning: unexpected NAN value was coerced to bool\n";

/// First byte available to PHP string literals in a command module.
pub(super) const COMMAND_DATA_END: u32 = COMMAND_DATA_BASE
    + ERR_DIV_ZERO.len() as u32
    + ERR_MOD_ZERO.len() as u32
    + ERR_NEG_SHIFT.len() as u32
    + ERR_INTDIV_OVERFLOW.len() as u32
    + ERR_WASI.len() as u32
    + ERR_OOM.len() as u32
    + ERR_HASH_APPEND_OCCUPIED.len() as u32
    + ERR_CALLABLE_DISPATCH.len() as u32
    + ERR_MIXED_HEAP_TYPE.len() as u32
    + ERR_UNCAUGHT_EXCEPTION.len() as u32
    + ERR_STR_REPEAT_NEGATIVE.len() as u32
    + ERR_STR_PAD_EMPTY.len() as u32
    + ERR_EXPLODE_EMPTY_SEP.len() as u32
    + ERR_STR_SPLIT_LENGTH.len() as u32
    + ERR_METHOD_CALL_PREFIX.len() as u32
    + ERR_METHOD_CALL_SUFFIX.len() as u32
    + PHP_TYPE_INT.len() as u32
    + PHP_TYPE_STRING.len() as u32
    + PHP_TYPE_FLOAT.len() as u32
    + PHP_TYPE_BOOL.len() as u32
    + PHP_TYPE_ARRAY.len() as u32
    + PHP_TYPE_NULL.len() as u32
    + PHP_TYPE_RESOURCE.len() as u32
    + PHP_TYPE_CALLABLE.len() as u32
    + PHP_TYPE_UNKNOWN.len() as u32
    + ERR_UNDEFINED_METHOD_PREFIX.len() as u32
    + ERR_UNDEFINED_METHOD_SEPARATOR.len() as u32
    + ERR_UNDEFINED_METHOD_SUFFIX.len() as u32
    + ERR_TOO_FEW_ARGS_PREFIX.len() as u32
    + ERR_TOO_FEW_ARGS_PASSED.len() as u32
    + ERR_TOO_FEW_ARGS_EXACTLY.len() as u32
    + ERR_TOO_FEW_ARGS_AT_LEAST.len() as u32
    + ERR_TOO_FEW_ARGS_SUFFIX.len() as u32
    + WARN_UNDEFINED_ARRAY_KEY_PREFIX.len() as u32
    + WARN_QUOTE.len() as u32
    + WARN_SUFFIX.len() as u32
    + crate::ir::ARRAY_OFFSET_ON_NULL_WARNING_PHP82.len() as u32
    + crate::ir::ARRAY_OFFSET_ON_NULL_WARNING.len() as u32
    + WARN_FLOAT_NOT_REPRESENTABLE_PREFIX.len() as u32
    + WARN_FLOAT_NOT_REPRESENTABLE_SUFFIX.len() as u32
    + WARN_NON_NUMERIC_VALUE.len() as u32
    + WARN_OBJECT_TO_SCALAR_PREFIX.len() as u32
    + WARN_OBJECT_TO_INT_SUFFIX.len() as u32
    + WARN_OBJECT_TO_FLOAT_SUFFIX.len() as u32
    + ERR_UNSUPPORTED_OPERAND.len() as u32
    + DEPRECATED_CHR_RANGE.len() as u32
    + DEPRECATED_ORD_LENGTH.len() as u32
    + WARN_ARRAY_TO_STRING.len() as u32
    + ERR_OBJECT_TO_STRING_PREFIX.len() as u32
    + ERR_OBJECT_TO_STRING_SUFFIX.len() as u32
    + DEPRECATED_FLOAT_TO_INT_PREFIX.len() as u32
    + DEPRECATED_FLOAT_STR_TO_INT_PREFIX.len() as u32
    + DEPRECATED_TO_INT_SUFFIX.len() as u32
    + DEPRECATED_STR_TO_INT_SUFFIX.len() as u32
    + ERR_RETURN_TYPE_PREFIX.len() as u32
    + ERR_RETURN_TYPE_MIDDLE.len() as u32
    + ERR_RETURN_TYPE_SUFFIX.len() as u32
    + PHP_CLASS_CLOSURE.len() as u32
    + ERR_RETURN_TYPE_SEPARATOR.len() as u32
    + ERR_ARGUMENT_TYPE_MIDDLE.len() as u32
    + ERR_ARGUMENT_NAME_PREFIX.len() as u32
    + ERR_ARGUMENT_TYPE_MUST_BE.len() as u32
    + ERR_ARGUMENT_TYPE_SUFFIX.len() as u32
    + DEPRECATED_ARGUMENT_NULL_PREFIX.len() as u32
    + DEPRECATED_ARGUMENT_NULL_MIDDLE.len() as u32
    + DEPRECATED_ARGUMENT_NULL_OF_TYPE.len() as u32
    + DEPRECATED_ARGUMENT_NULL_SUFFIX.len() as u32
    + WARN_FOREACH_NON_ITERABLE_PREFIX.len() as u32
    + WARN_FOREACH_NON_ITERABLE_SUFFIX.len() as u32
    + ERR_OPERAND_TYPES_PREFIX.len() as u32
    + ERR_OPERAND_TYPES_SPACE.len() as u32
    + WARN_NAN_TO_STRING.len() as u32
    + WARN_NAN_TO_BOOL.len() as u32
    + PHP_VALUE_TRUE.len() as u32
    + PHP_VALUE_FALSE.len() as u32
    + ERR_COUNT_PREFIX.len() as u32
    + ERR_COUNT_SUFFIX.len() as u32
    + WARN_UNINIT_STRING_OFFSET.len() as u32
    + WARN_OFFSET_NEWLINE.len() as u32
    + WARN_PROPERTY_ON_NULL_PREFIX.len() as u32
    + WARN_PROPERTY_ON_NULL_SUFFIX.len() as u32
    + WARN_OFFSET_ON_TRUE.len() as u32
    + WARN_OFFSET_ON_FALSE.len() as u32
    + WARN_OFFSET_ON_INT.len() as u32
    + WARN_OFFSET_ON_FLOAT.len() as u32
    + WARN_OFFSET_ON_TYPE_BOOL.len() as u32
    + WARN_OFFSET_ON_TYPE_INT.len() as u32
    + WARN_OFFSET_ON_TYPE_FLOAT.len() as u32
    + WARN_FOPEN_FAILED.len() as u32
    + WARN_FILE_GET_CONTENTS_FAILED.len() as u32;

/// Adds the import-free runtime every module needs: the compatibility concat
/// cursor global and the heap-backed `__rt_concat` helper.
pub(super) fn emit_common_runtime(wm: &mut WatModule) {
    wm.add_global(Global {
        name: "__concat_off".to_string(),
        ty: ValType::I32,
        mutable: true,
        init: CONCAT_BASE as i64,
    });
    wm.add_raw_func(RT_CONCAT);
}

/// Adds the WASI imports and `__rt_*` helpers a command (main-bearing) module needs.
///
/// Imports `proc_exit` and `fd_write` from `wasi_snapshot_preview1` and registers
/// the echo helpers. Must be called before functions that reference these symbols
/// are rendered.
pub(super) fn emit_command_runtime(wm: &mut WatModule) {
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "proc_exit".to_string(),
        internal: "wasi_proc_exit".to_string(),
        params: vec![ValType::I32],
        results: vec![],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "fd_write".to_string(),
        internal: "wasi_fd_write".to_string(),
        // fd, iovs_ptr, iovs_len, nwritten_ptr -> errno
        params: vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "fd_read".to_string(),
        internal: "wasi_fd_read".to_string(),
        // fd, iovs_ptr, iovs_len, nread_ptr -> errno
        params: vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "fd_seek".to_string(),
        internal: "wasi_fd_seek".to_string(),
        // fd, offset, whence, newoffset_ptr -> errno
        params: vec![ValType::I32, ValType::I64, ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    // The path family. WASI Preview 1 is capability-based: every one of these takes the fd of a
    // directory the host preopened, which `__rt_wasi_dirfd` finds by probing from fd 3.
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "fd_prestat_get".to_string(),
        internal: "wasi_fd_prestat_get".to_string(),
        // fd, prestat_buf -> errno
        params: vec![ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "fd_close".to_string(),
        internal: "wasi_fd_close".to_string(),
        params: vec![ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "path_open".to_string(),
        internal: "wasi_path_open".to_string(),
        // dirfd, dirflags, path, path_len, oflags, rights_base, rights_inheriting,
        // fdflags, opened_fd_out -> errno
        params: vec![
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I64,
            ValType::I64,
            ValType::I32,
            ValType::I32,
        ],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "path_filestat_get".to_string(),
        internal: "wasi_path_filestat_get".to_string(),
        // dirfd, flags, path, path_len, filestat_buf -> errno
        params: vec![
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
        ],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "path_unlink_file".to_string(),
        internal: "wasi_path_unlink_file".to_string(),
        // dirfd, path, path_len -> errno
        params: vec![ValType::I32, ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "args_sizes_get".to_string(),
        internal: "wasi_args_sizes_get".to_string(),
        // argc_ptr, argv_buf_size_ptr -> errno
        params: vec![ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    wm.import_func(FuncImport {
        module: "wasi_snapshot_preview1".to_string(),
        field: "args_get".to_string(),
        internal: "wasi_args_get".to_string(),
        // argv_ptr_array, argv_buf -> errno
        params: vec![ValType::I32, ValType::I32],
        results: vec![ValType::I32],
    });
    emit_failure_runtime(wm);
    wm.add_raw_func(RT_WASI_WRITE_ALL);
    wm.add_raw_func(RT_WASI_WRITE_OR_FAIL);
    wm.add_raw_func(RT_ECHO_I64);
    wm.add_raw_func(RT_ECHO_F64);
    wm.add_raw_func(RT_ECHO_STR);
    wm.add_raw_func(RT_ECHO_BOOL);
    wm.add_raw_func(RT_ARGC);
    wm.add_raw_func(RT_STRLEN_C);
    wm.add_raw_func(RT_ARGV);
    wm.add_raw_func(RT_MIXED_WRITE_STDOUT);
    wm.add_raw_func(RT_READLINE);
    super::files::emit_file_runtime(wm);
}

/// `__rt_readline`: reads one line from stdin, WITHOUT its terminating newline.
///
/// Two details come from measuring php-src 8.5.6 rather than from the native backend, which
/// gets both wrong (`printf 'Ada\n' | ...` answers `Hello Ada\n!` there, and prints the prompt):
///
/// - The newline is NOT part of the result. `readline()` strips it; keeping it puts a line break
///   in the middle of `"Hello " . $name . "!"`.
/// - The PROMPT is not written to stdout. php-src hands it to the terminal, so with stdout
///   redirected it does not appear at all — writing it here would add bytes php-src never emits.
///   The argument is therefore accepted and ignored, which is exactly what a captured run sees.
///
/// Bytes land in the legacy concat reservation, which is dead space kept only so static-data
/// offsets stay stable, so nothing else can be occupying it. Reading one byte at a time keeps the
/// helper free of any buffering that could swallow input a later read needs.
///
/// EOF with nothing read answers the empty string. php-src answers `false` there, which the EIR's
/// `Str` result cannot carry — the same dropped-null shape as elsewhere, not a choice made here.
const RT_READLINE: &str = r#"(func $__rt_readline (param $prompt_ptr i32) (param $prompt_len i64) (result i32) (result i64)
  (local $n i32)
  (local $rc i32)
  (block $done (loop $next
    (br_if $done (i32.ge_u (local.get $n) (i32.const 65535)))     ;; bound by the reservation
    (i32.store (i32.const 0) (i32.add (i32.const 64) (local.get $n)))  ;; iovec buf = base + n
    (i32.store (i32.const 4) (i32.const 1))                       ;; iovec len = 1 byte
    (local.set $rc (call $wasi_fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 8)))
    (br_if $done (i32.ne (local.get $rc) (i32.const 0)))          ;; read error -> end the line
    (br_if $done (i32.eqz (i32.load (i32.const 8))))              ;; 0 bytes read = EOF
    (br_if $done (i32.eq (i32.load8_u (i32.add (i32.const 64) (local.get $n))) (i32.const 10)))  ;; newline ends it and is dropped
    (local.set $n (i32.add (local.get $n) (i32.const 1)))
    (br $next)))
  (call $__rt_str_persist (i32.const 64) (i64.extend_i32_u (local.get $n))))
"#;

/// Emits immutable diagnostic data and the command-runtime failure dispatcher.
///
/// Error code 1 is division by zero, 2 modulo by zero, 3 a negative shift,
/// 4 `PHP_INT_MIN / -1` for integer division, 5 a WASI boundary failure, and
/// 6 allocator exhaustion or arithmetic overflow, 7 an occupied saturated
/// array append key, 8 a rejected callable dispatch, 9 a runtime Mixed
/// heap-kind mismatch, and 10 a PHP exception that reached the top of `main`
/// uncaught.
/// The helper writes the selected message to stderr, exits with status 255, and
/// ends in `unreachable` so validation does not treat `proc_exit` as returning.
/// The same data region also owns the warning fragments used by the non-fatal
/// undefined-index diagnostic.
fn emit_failure_runtime(wm: &mut WatModule) {
    let fixed_messages = [
        ERR_DIV_ZERO,
        ERR_MOD_ZERO,
        ERR_NEG_SHIFT,
        ERR_INTDIV_OVERFLOW,
        ERR_WASI,
        ERR_OOM,
        ERR_HASH_APPEND_OCCUPIED,
        ERR_CALLABLE_DISPATCH,
        ERR_MIXED_HEAP_TYPE,
        ERR_UNCAUGHT_EXCEPTION,
        ERR_STR_REPEAT_NEGATIVE,
        ERR_STR_PAD_EMPTY,
        ERR_EXPLODE_EMPTY_SEP,
        ERR_STR_SPLIT_LENGTH,
    ];
    let method_messages = [
        ERR_METHOD_CALL_PREFIX,
        ERR_METHOD_CALL_SUFFIX,
        PHP_TYPE_INT,
        PHP_TYPE_STRING,
        PHP_TYPE_FLOAT,
        PHP_TYPE_BOOL,
        PHP_TYPE_ARRAY,
        PHP_TYPE_NULL,
        PHP_TYPE_RESOURCE,
        PHP_TYPE_CALLABLE,
        PHP_TYPE_UNKNOWN,
        ERR_UNDEFINED_METHOD_PREFIX,
        ERR_UNDEFINED_METHOD_SEPARATOR,
        ERR_UNDEFINED_METHOD_SUFFIX,
        // Appended LAST so every index the emitters above already use keeps its message.
        ERR_TOO_FEW_ARGS_PREFIX,
        ERR_TOO_FEW_ARGS_PASSED,
        ERR_TOO_FEW_ARGS_EXACTLY,
        ERR_TOO_FEW_ARGS_AT_LEAST,
        ERR_TOO_FEW_ARGS_SUFFIX,
    ];
    let warning_messages = [
        WARN_UNDEFINED_ARRAY_KEY_PREFIX,
        WARN_QUOTE,
        WARN_SUFFIX,
        crate::ir::ARRAY_OFFSET_ON_NULL_WARNING_PHP82.as_bytes(),
        crate::ir::ARRAY_OFFSET_ON_NULL_WARNING.as_bytes(),
        WARN_FLOAT_NOT_REPRESENTABLE_PREFIX,
        WARN_FLOAT_NOT_REPRESENTABLE_SUFFIX,
        WARN_NON_NUMERIC_VALUE,
        ERR_UNSUPPORTED_OPERAND,
        WARN_OBJECT_TO_SCALAR_PREFIX,
        WARN_OBJECT_TO_INT_SUFFIX,
        WARN_OBJECT_TO_FLOAT_SUFFIX,
        DEPRECATED_CHR_RANGE,
        DEPRECATED_ORD_LENGTH,
        WARN_ARRAY_TO_STRING,
        ERR_OBJECT_TO_STRING_PREFIX,
        ERR_OBJECT_TO_STRING_SUFFIX,
        // The declared-return coercion fragments come LAST so every index already in use
        // above keeps pointing at the same message.
        DEPRECATED_FLOAT_TO_INT_PREFIX,
        DEPRECATED_FLOAT_STR_TO_INT_PREFIX,
        DEPRECATED_TO_INT_SUFFIX,
        DEPRECATED_STR_TO_INT_SUFFIX,
        ERR_RETURN_TYPE_PREFIX,
        ERR_RETURN_TYPE_MIDDLE,
        ERR_RETURN_TYPE_SUFFIX,
        PHP_CLASS_CLOSURE,
        ERR_RETURN_TYPE_SEPARATOR,
        WARN_NAN_TO_STRING,
        WARN_NAN_TO_BOOL,
        PHP_VALUE_TRUE,
        PHP_VALUE_FALSE,
        ERR_COUNT_PREFIX,
        ERR_COUNT_SUFFIX,
        WARN_UNINIT_STRING_OFFSET,
        WARN_OFFSET_NEWLINE,
        // Appended LAST, like the return-coercion fragments above: every index already in use
        // keeps pointing at the same message, so the positional slices below do not move.
        WARN_PROPERTY_ON_NULL_PREFIX,
        WARN_PROPERTY_ON_NULL_SUFFIX,
        WARN_OFFSET_ON_TRUE,
        WARN_OFFSET_ON_FALSE,
        WARN_OFFSET_ON_INT,
        WARN_OFFSET_ON_FLOAT,
        WARN_OFFSET_ON_TYPE_BOOL,
        WARN_OFFSET_ON_TYPE_INT,
        WARN_OFFSET_ON_TYPE_FLOAT,
        WARN_FOPEN_FAILED,
        WARN_FILE_GET_CONTENTS_FAILED,
        // Appended LAST for the same reason as every group above it.
        ERR_ARGUMENT_TYPE_MIDDLE,
        ERR_ARGUMENT_NAME_PREFIX,
        ERR_ARGUMENT_TYPE_MUST_BE,
        ERR_ARGUMENT_TYPE_SUFFIX,
        DEPRECATED_ARGUMENT_NULL_PREFIX,
        DEPRECATED_ARGUMENT_NULL_MIDDLE,
        DEPRECATED_ARGUMENT_NULL_OF_TYPE,
        DEPRECATED_ARGUMENT_NULL_SUFFIX,
        WARN_FOREACH_NON_ITERABLE_PREFIX,
        WARN_FOREACH_NON_ITERABLE_SUFFIX,
        ERR_OPERAND_TYPES_PREFIX,
        ERR_OPERAND_TYPES_SPACE,
    ];
    let mut offsets = Vec::with_capacity(fixed_messages.len());
    let mut cursor = COMMAND_DATA_BASE;
    for message in fixed_messages {
        offsets.push((cursor, message.len() as u32));
        wm.add_data(DataSegment {
            offset: cursor,
            bytes: message.to_vec(),
        });
        cursor += message.len() as u32;
    }
    let mut method_offsets = Vec::with_capacity(method_messages.len());
    for message in method_messages {
        method_offsets.push((cursor, message.len() as u32));
        wm.add_data(DataSegment {
            offset: cursor,
            bytes: message.to_vec(),
        });
        cursor += message.len() as u32;
    }
    let mut warning_offsets = Vec::with_capacity(warning_messages.len());
    for message in warning_messages {
        warning_offsets.push((cursor, message.len() as u32));
        wm.add_data(DataSegment {
            offset: cursor,
            bytes: message.to_vec(),
        });
        cursor += message.len() as u32;
    }
    debug_assert_eq!(cursor, COMMAND_DATA_END);

    let mut wat = String::from(
        "(func $__rt_fail (param $code i32)\n  (local $ptr i32) (local $len i32)\n",
    );
    for (index, (offset, len)) in offsets.iter().enumerate() {
        wat.push_str(&format!(
            "  (if (i32.eq (local.get $code) (i32.const {}))\n    (then\n      (local.set $ptr (i32.const {}))\n      (local.set $len (i32.const {}))))\n",
            index + 1,
            offset,
            len
        ));
    }
    wat.push_str(
        "  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $ptr) (local.get $len)))\n  (call $wasi_proc_exit (i32.const 255))\n  unreachable ;; elephc-trap:post-noreturn:runtime-fatal-exit\n)",
    );
    wm.add_raw_func(&wat);
    emit_method_call_failure_runtime(wm, &method_offsets);
    emit_undefined_array_key_warning_runtime(wm, &warning_offsets[..17]);
    emit_return_coercion_runtime(wm, &warning_offsets[17..34], &method_offsets[2..11]);
    emit_property_on_null_warning_runtime(wm, &warning_offsets[34..36]);
    emit_offset_on_scalar_warning_runtime(wm, &warning_offsets[36..43]);
    emit_uninit_string_offset_warning_runtime(wm, &warning_offsets[32..34]);
    emit_open_failure_warning_runtime(wm, &warning_offsets[43..45]);
    emit_argument_coercion_runtime(
        wm,
        &warning_offsets[45..53],
        warning_offsets[21],
        warning_offsets[25],
        &method_offsets[2..11],
    );
    emit_foreach_warning_runtime(wm, &warning_offsets[53..55]);
    emit_arithmetic_coercion_runtime(
        wm,
        &warning_offsets[55..57],
        &warning_offsets[5..7],
        warning_offsets[33],
    );
}

/// Emits the warning `foreach` produces for a value that is neither an array nor an object.
///
/// Measured on php-src 8.5.6: it WARNS, names the type that arrived, and runs the body zero
/// times rather than raising. The type word comes from the shared `__rt_type_word_for_tag`, so
/// an object would contribute its class — unreachable here, since an object IS iterable and
/// never reaches this arm.
fn emit_foreach_warning_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 2);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (suffix_ptr, suffix_len) = offsets[1];
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_foreach_non_iterable (param $tag i64) (param $lo i64)
  (local $word_ptr i32) (local $word_len i32)
  (call $__rt_type_word_for_tag (local.get $tag) (local.get $lo))
  (local.set $word_len)
  (local.set $word_ptr)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $word_ptr) (local.get $word_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))"#
    ));
}

/// Emits the coercion php-src performs when a boxed value reaches an internal function's
/// declared `string` parameter.
///
/// Measured on php-src 8.5.6 for `strtoupper($mixed)`, one arm per runtime tag: a string, int,
/// float or bool converts EXACTLY as `(string)` does, which is why the scalar arms delegate to
/// `__rt_mixed_cast_string` rather than restating it; `null` converts to `""` but raises a
/// `Deprecated` first; and an array, object, resource or closure does not convert at all — it is
/// a `TypeError` naming what arrived, where `(string)` of the same array would have produced
/// `"Array"` with a warning. NaN warns on the way through, exactly as the declared-RETURN
/// coercion already does.
///
/// The function and parameter names travel from the call site as `(ptr, len)` pairs, the same
/// way `__rt_fail_return_type` carries its function name, so no per-call-site data layout is
/// needed beyond the two interned strings.
fn emit_argument_coercion_runtime(
    wm: &mut WatModule,
    offsets: &[(u32, u32)],
    error_prefix: (u32, u32),
    separator: (u32, u32),
    type_offsets: &[(u32, u32)],
) {
    debug_assert_eq!(offsets.len(), 8);
    debug_assert_eq!(type_offsets.len(), 9);
    let (argument_ptr, argument_len) = offsets[0];
    let (name_ptr, name_len) = offsets[1];
    let (must_be_ptr, must_be_len) = offsets[2];
    let (given_ptr, given_len) = offsets[3];
    let (deprecated_ptr, deprecated_len) = offsets[4];
    let (passing_ptr, passing_len) = offsets[5];
    let (of_type_ptr, of_type_len) = offsets[6];
    let (is_deprecated_ptr, is_deprecated_len) = offsets[7];
    // The word sits mid-sentence here, so its trailing newline is dropped from the length —
    // the same adjustment the declared-return fatal makes.
    let (string_word_ptr, string_word_len) = (type_offsets[1].0, type_offsets[1].1 - 1);
    let (error_prefix_ptr, error_prefix_len) = error_prefix;
    let (separator_ptr, separator_len) = separator;

    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_argument_type (param $fn_ptr i32) (param $fn_len i32) (param $param_ptr i32) (param $param_len i32) (param $argno i64) (param $target_ptr i32) (param $target_len i32) (param $tag i64) (param $lo i64)
  (local $word_ptr i32) (local $word_len i32) (local $num_ptr i32) (local $num_len i32)
  (call $__rt_type_word_for_tag (local.get $tag) (local.get $lo))
  (local.set $word_len)
  (local.set $word_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {error_prefix_ptr}) (i32.const {error_prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $fn_ptr) (local.get $fn_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {argument_ptr}) (i32.const {argument_len})))
  (call $__rt_itoa (local.get $argno) (global.get $__float_scratch))
  (local.set $num_len)
  (local.set $num_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $num_ptr) (local.get $num_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {name_ptr}) (i32.const {name_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $param_ptr) (local.get $param_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {must_be_ptr}) (i32.const {must_be_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $target_ptr) (local.get $target_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {separator_ptr}) (i32.const {separator_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $word_ptr) (local.get $word_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {given_ptr}) (i32.const {given_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:argument-type-fatal-exit
)"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_deprecate_argument_null (param $fn_ptr i32) (param $fn_len i32) (param $param_ptr i32) (param $param_len i32) (param $argno i64) (param $target_ptr i32) (param $target_len i32)
  (local $num_ptr i32) (local $num_len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {deprecated_ptr}) (i32.const {deprecated_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $fn_ptr) (local.get $fn_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {passing_ptr}) (i32.const {passing_len}))
  (call $__rt_itoa (local.get $argno) (global.get $__float_scratch))
  (local.set $num_len)
  (local.set $num_ptr)
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $num_ptr) (local.get $num_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {name_ptr}) (i32.const {name_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $param_ptr) (local.get $param_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {of_type_ptr}) (i32.const {of_type_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $target_ptr) (local.get $target_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {is_deprecated_ptr}) (i32.const {is_deprecated_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_arg_string (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (param $param_ptr i32) (param $param_len i32) (param $argno i64) (result i32) (result i32)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $f f64) (local $ok i32) (local $sptr i32) (local $slen i32) (local $pptr i32) (local $plen i64)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eq (local.get $tag) (i64.const 2))                     ;; NaN warns on the way through
    (then
      (local.set $f (f64.reinterpret_i64 (local.get $lo)))
      (if (f64.ne (local.get $f) (local.get $f))
        (then (call $__rt_deprecate_nan_to_string)))))
  (if (i64.le_u (local.get $tag) (i64.const 3))                   ;; int/string/float/bool convert as `(string)` does
    (then (return (call $__rt_mixed_cast_string (local.get $cell)))))
  (if (i64.eq (local.get $tag) (i64.const 8))                     ;; null still converts, after a deprecation
    (then
      (call $__rt_deprecate_argument_null (local.get $fn_ptr) (local.get $fn_len) (local.get $param_ptr) (local.get $param_len) (local.get $argno) (i32.const {string_word_ptr}) (i32.const {string_word_len}))
      (return (i32.const 0) (i32.const 0))))
  (if (i64.eq (local.get $tag) (i64.const 6))                     ;; an object with `__toString` CONVERTS
    (then
      (call $__rt_object_to_string (i32.wrap_i64 (local.get $lo)))
      (local.set $ok)
      (local.set $slen)
      (local.set $sptr)
      (if (local.get $ok)
        (then
          (call $__rt_str_persist (local.get $sptr) (i64.extend_i32_u (local.get $slen)))  ;; own an independent copy
          (local.set $plen)
          (local.set $pptr)
          (call $__rt_decref_any (local.get $sptr))               ;; a callee's Str return is OWNED
          (return (local.get $pptr) (i32.wrap_i64 (local.get $plen)))))))
  (call $__rt_fail_argument_type (local.get $fn_ptr) (local.get $fn_len) (local.get $param_ptr) (local.get $param_len) (local.get $argno) (i32.const {string_word_ptr}) (i32.const {string_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:argument-coerce-tostring
"#
    ));
}

/// Emits the coercion PHP performs on a BOXED operand of an integer arithmetic operator.
///
/// This is a THIRD contract, distinct from both the declared-return and the declared-parameter
/// ones, and every difference was measured on php-src 8.5.6 with `$mixed % 3`:
///
/// - `null` is silently 0 — a parameter deprecates there and a return raises;
/// - a non-numeric string is `Unsupported operand types: string % int`, naming the operand types
///   rather than `must be of type int`;
/// - `INF`/`NAN` do NOT raise: they warn `The float INF is not representable as an int, cast
///   occurred` and yield 0, where a parameter raises a `TypeError`.
///
/// What IS shared is the numeric middle: a lost fraction deprecates identically, from a float
/// and from a float-shaped string alike, so those two notices come from the same helpers the
/// other boundaries call.
fn emit_arithmetic_coercion_runtime(
    wm: &mut WatModule,
    offsets: &[(u32, u32)],
    float_warning: &[(u32, u32)],
    newline: (u32, u32),
) {
    debug_assert_eq!(offsets.len(), 2);
    debug_assert_eq!(float_warning.len(), 2);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (space_ptr, space_len) = offsets[1];
    let (not_repr_prefix_ptr, not_repr_prefix_len) = float_warning[0];
    let (not_repr_suffix_ptr, not_repr_suffix_len) = float_warning[1];
    let (newline_ptr, newline_len) = newline;
    let value_offset = super::mixed_numeric::CLASS_VALUE_OFFSET;

    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_operand_types (param $tag i64) (param $lo i64) (param $op_ptr i32) (param $op_len i32) (param $right_ptr i32) (param $right_len i32)
  (local $word_ptr i32) (local $word_len i32)
  (call $__rt_type_word_for_tag (local.get $tag) (local.get $lo))
  (local.set $word_len)
  (local.set $word_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $word_ptr) (local.get $word_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {space_ptr}) (i32.const {space_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $op_ptr) (local.get $op_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {space_ptr}) (i32.const {space_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $right_ptr) (local.get $right_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {newline_ptr}) (i32.const {newline_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:operand-types-fatal-exit
)"#
    ));

    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_arith_int (param $cell i32) (param $op_ptr i32) (param $op_len i32) (param $right_ptr i32) (param $right_len i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $f f64) (local $t f64) (local $cls i32) (local $fptr i32) (local $flen i32)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eqz (local.get $tag))                                  ;; int
    (then (return (local.get $lo))))
  (if (i64.eq (local.get $tag) (i64.const 3))                     ;; bool: stored normalized 0/1
    (then (return (local.get $lo))))
  (if (i64.eq (local.get $tag) (i64.const 8))                     ;; null is SILENTLY zero here
    (then (return (i64.const 0))))
  (if (i64.eq (local.get $tag) (i64.const 2))                     ;; float
    (then
      (local.set $f (f64.reinterpret_i64 (local.get $lo)))
      (if (i32.and
            (f64.eq (local.get $f) (local.get $f))
            (i32.and
              (f64.ge (local.get $f) (f64.const -9223372036854775808))
              (f64.lt (local.get $f) (f64.const 9223372036854775808))))
        (then
          (local.set $t (f64.trunc (local.get $f)))
          (if (f64.ne (local.get $t) (local.get $f))
            (then (call $__rt_deprecate_return_float_to_int (local.get $lo))))
          (return (i64.trunc_f64_s (local.get $t)))))
      ;; Out of range or not a number: php-src WARNS and uses 0 rather than raising.
      (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {not_repr_prefix_ptr}) (i32.const {not_repr_prefix_len}))
      (call $__rt_ftoa (local.get $lo) (i32.add (global.get $__float_scratch) (i32.const 1024)) (i32.const 80) (i32.add (global.get $__float_scratch) (i32.const 2048)) (i32.const 792) (i32.add (global.get $__float_scratch) (i32.const 4096)))
      (local.set $flen)                                           ;; ftoa returns (ptr, len), len on top
      (local.set $fptr)
      (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $fptr) (local.get $flen))
      (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {not_repr_suffix_ptr}) (i32.const {not_repr_suffix_len}))
      (return (i64.const 0))))
  (if (i64.eq (local.get $tag) (i64.const 1))                     ;; string
    (then
      (local.set $cls (call $__rt_str_numeric_class (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi))))
      (if (i32.eq (local.get $cls) (i32.const 1))
        (then (return (i64.load (i32.add (global.get $__float_scratch) (i32.const {value_offset}))))))
      (if (i32.eq (local.get $cls) (i32.const 2))
        (then
          (local.set $f (f64.load (i32.add (global.get $__float_scratch) (i32.const {value_offset}))))
          (local.set $t (f64.trunc (local.get $f)))
          (if (f64.ne (local.get $t) (local.get $f))
            (then (call $__rt_deprecate_return_str_to_int (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi)))))
          (return (i64.trunc_f64_s (local.get $t)))))))
  (call $__rt_fail_operand_types (local.get $tag) (local.get $lo) (local.get $op_ptr) (local.get $op_len) (local.get $right_ptr) (local.get $right_len))
  unreachable)                                                    ;; elephc-trap:post-noreturn:arith-coerce-int
"#
    ));
}

/// Emits php-src's warning for a property read whose receiver is null.
///
/// A raw object pointer can be 0 since a missed `array<Object>` element reads as null with the
/// element's own non-null type, and PHP names the property rather than just the receiver:
/// `Warning: Attempt to read property "age" on null`. The name arrives as a (pointer, length)
/// pair from the instruction's own interned string, so no per-property data segment is needed.
fn emit_property_on_null_warning_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 2);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (suffix_ptr, suffix_len) = offsets[1];
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_property_on_null (param $name_ptr i32) (param $name_len i32)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $name_ptr) (local.get $name_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len}))))"#
    ));
}

/// Emits the warning a read of `$scalar[$key]` produces, dispatched on the runtime tag.
///
/// PHP answers null for every non-container receiver, but it says so first, and WHAT it says
/// depends on the version profile the module was compiled for — the same split the null
/// receiver already carries. Before 8.3 the message names the TYPE for all three; from 8.3 it
/// names the type for int and float and the VALUE for a boolean, so `true` and `false` are two
/// distinct messages there and one shared message before.
///
/// The tag is the Mixed cell's own: 0 int, 2 float, 3 bool. Any other tag reaches this only if a
/// caller mis-dispatched, and writes nothing rather than a message for the wrong type.
fn emit_offset_on_scalar_warning_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 7);
    let php83_or_later = crate::codegen_support::runtime::array_offset_on_null_warning()
        == crate::ir::ARRAY_OFFSET_ON_NULL_WARNING;
    let (true_ptr, true_len) = if php83_or_later { offsets[0] } else { offsets[4] };
    let (false_ptr, false_len) = if php83_or_later { offsets[1] } else { offsets[4] };
    let (int_ptr, int_len) = if php83_or_later { offsets[2] } else { offsets[5] };
    let (float_ptr, float_len) = if php83_or_later { offsets[3] } else { offsets[6] };
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_array_offset_on_scalar (param $tag i64) (param $lo i64)
  (if (i64.eq (local.get $tag) (i64.const 0))                     ;; int receiver
    (then (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {int_ptr}) (i32.const {int_len})) (return)))
  (if (i64.eq (local.get $tag) (i64.const 2))                     ;; float receiver
    (then (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {float_ptr}) (i32.const {float_len})) (return)))
  (if (i64.eq (local.get $tag) (i64.const 3))                     ;; bool receiver: the VALUE names it from 8.3
    (then
      (if (i64.eqz (local.get $lo))
        (then (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {false_ptr}) (i32.const {false_len})))
        (else (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {true_ptr}) (i32.const {true_len})))))))"#
    ));
}

/// Emits the warnings `fopen` and `file_get_contents` produce for a path they cannot open.
///
/// Both answer `false` afterwards, so the value is right either way and this is purely the
/// diagnostic. The wording is the native backend's, which stops short of php-src's path and
/// errno detail; matching native keeps the two Elephc targets in agreement.
fn emit_open_failure_warning_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 2);
    let (fopen_ptr, fopen_len) = offsets[0];
    let (get_ptr, get_len) = offsets[1];
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_fopen_failed
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {fopen_ptr}) (i32.const {fopen_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_file_get_contents_failed
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {get_ptr}) (i32.const {get_len})))"#
    ));
}

/// Emits the warning a string offset outside the string produces.
///
/// The index is written AS GIVEN — php-src reports a negative one negative, without resolving it
/// from the end first — and the read answers the empty string afterwards.
fn emit_uninit_string_offset_warning_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 2);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (newline_ptr, newline_len) = offsets[1];
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_uninit_string_offset (param $index i64)
  (local $ptr i32) (local $len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len}))
  (call $__rt_itoa (local.get $index) (global.get $__float_scratch))
  (local.set $len)
  (local.set $ptr)
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (local.get $len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {newline_ptr}) (i32.const {newline_len})))"#
    ));
}

/// Emits the fatal path used when a `Mixed` receiver is not an object.
///
/// The helper composes the PHP-visible method name with the runtime Mixed tag,
/// writes the diagnostic to stderr, and terminates with PHP's fatal status 255.
fn emit_method_call_failure_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 19);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (suffix_ptr, suffix_len) = offsets[1];
    let type_offsets = &offsets[2..11];
    let mut wat = format!(
        "(func $__rt_fail_method_call_non_object (param $method_ptr i32) (param $method_len i32) (param $tag i32)\n  (local $type_ptr i32) (local $type_len i32)\n  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len})))\n  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $method_ptr) (local.get $method_len)))\n  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))\n  (local.set $type_ptr (i32.const {}))\n  (local.set $type_len (i32.const {}))\n",
        type_offsets[8].0, type_offsets[8].1
    );
    for (tag, type_index) in [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 4),
        (8, 5),
        (9, 6),
        (10, 7),
    ] {
        let (type_ptr, type_len) = type_offsets[type_index];
        wat.push_str(&format!(
            "  (if (i32.eq (local.get $tag) (i32.const {tag}))\n    (then\n      (local.set $type_ptr (i32.const {type_ptr}))\n      (local.set $type_len (i32.const {type_len}))))\n"
        ));
    }
    wat.push_str(
        "  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $type_ptr) (local.get $type_len)))\n  (call $wasi_proc_exit (i32.const 255))\n  unreachable ;; elephc-trap:post-noreturn:method-type-fatal-exit\n)",
    );
    wm.add_raw_func(&wat);
    emit_undefined_method_failure_runtime(wm, &offsets[11..14]);
    emit_too_few_arguments_failure_runtime(wm, &offsets[12..13], &offsets[14..19]);
}

/// Emits the fatal a dynamic dispatch raises when php-src would not enter the selected class.
///
/// The runtime class-name table supplies the concrete class exactly as the undefined-method
/// fatal does — reusing its `::` separator — and the two counts are rendered through the shared
/// signed `__rt_itoa`. `exact` picks between php-src's two wordings; the caller knows which
/// applies because the signature is closed-world.
fn emit_too_few_arguments_failure_runtime(
    wm: &mut WatModule,
    separator: &[(u32, u32)],
    offsets: &[(u32, u32)],
) {
    debug_assert_eq!(separator.len(), 1);
    debug_assert_eq!(offsets.len(), 5);
    let (separator_ptr, separator_len) = separator[0];
    let (prefix_ptr, prefix_len) = offsets[0];
    let (passed_ptr, passed_len) = offsets[1];
    let (exactly_ptr, exactly_len) = offsets[2];
    let (at_least_ptr, at_least_len) = offsets[3];
    let (suffix_ptr, suffix_len) = offsets[4];
    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_too_few_arguments (param $cid i64) (param $method_ptr i32) (param $method_len i32) (param $passed i64) (param $expected i64) (param $exact i32)
  (local $class_ptr i32) (local $class_len i64) (local $num_ptr i32) (local $num_len i32)
  (call $__rt_class_name_by_cid (local.get $cid))
  (local.set $class_len)
  (local.set $class_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $class_ptr) (i32.wrap_i64 (local.get $class_len))))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {separator_ptr}) (i32.const {separator_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $method_ptr) (local.get $method_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {passed_ptr}) (i32.const {passed_len})))
  (call $__rt_itoa (local.get $passed) (global.get $__float_scratch))
  (local.set $num_len)
  (local.set $num_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $num_ptr) (local.get $num_len)))
  (if (local.get $exact)
    (then (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {exactly_ptr}) (i32.const {exactly_len}))))
    (else (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {at_least_ptr}) (i32.const {at_least_len})))))
  (call $__rt_itoa (local.get $expected) (global.get $__float_scratch))
  (local.set $num_len)
  (local.set $num_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $num_ptr) (local.get $num_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:too-few-arguments-fatal-exit
)"#
    ));
}

/// Emits the runtime behind PHP's IMPLICIT coercion at a declared `int` return.
///
/// PHP does NOT convert there the way `(int)` does. Measured on php-src 8.5.6 and validated
/// against a 1200-value random sweep: an int or bool passes through; a float in i64's range
/// truncates, and a LOST FRACTION is a `Deprecated`, not a failure; a WHOLLY numeric string
/// converts by the same rules; and everything else — null, a leading-numeric or non-numeric
/// string, an out-of-range or non-finite float, a container, an object — is a `TypeError`
/// naming the function and the type that arrived.
///
/// The message needs no interned per-function text: the function name travels as `(ptr, len)`
/// from the call site, exactly as the non-object method-call fatal already does. An object
/// contributes its CLASS name rather than the word "object", which is why this helper reaches
/// for the class-name table like `__rt_fail_undefined_method` does.
///
/// The `TypeError` is raised as a deterministic fatal rather than a catchable throw: a raise
/// site composing its message at runtime cannot go through `emit_runtime_error_throw`, which
/// resolves a STATIC message from `default_strings`. The text and the 255 exit status are
/// PHP's; only a `catch (TypeError)` around the call would notice the difference.
fn emit_return_coercion_runtime(
    wm: &mut WatModule,
    offsets: &[(u32, u32)],
    type_offsets: &[(u32, u32)],
) {
    debug_assert_eq!(offsets.len(), 17);
    debug_assert_eq!(type_offsets.len(), 9);
    let (float_prefix_ptr, float_prefix_len) = offsets[0];
    let (str_prefix_ptr, str_prefix_len) = offsets[1];
    let (float_suffix_ptr, float_suffix_len) = offsets[2];
    let (str_suffix_ptr, str_suffix_len) = offsets[3];
    let (err_prefix_ptr, err_prefix_len) = offsets[4];
    let (err_middle_ptr, err_middle_len) = offsets[5];
    let (err_suffix_ptr, err_suffix_len) = offsets[6];
    let (closure_ptr, closure_len) = offsets[7];
    let (err_sep_ptr, err_sep_len) = offsets[8];
    let (nan_string_ptr, nan_string_len) = offsets[9];
    let (nan_bool_ptr, nan_bool_len) = offsets[10];
    let (true_ptr, true_len) = offsets[11];
    let (false_ptr, false_len) = offsets[12];
    let (count_prefix_ptr, count_prefix_len) = offsets[13];
    let (count_suffix_ptr, count_suffix_len) = offsets[14];
    let (uninit_offset_ptr, uninit_offset_len) = offsets[15];
    let (newline_ptr, newline_len) = offsets[16];
    // The shared `PHP_TYPE_*` words each end with the newline that terminates the method-call
    // fatal; here a word sits mid-sentence, so the newline is dropped from the length.
    let word = |index: usize| -> (u32, u32) {
        let (ptr, len) = type_offsets[index];
        debug_assert!(len > 1, "a type word is at least one byte plus its newline");
        (ptr, len - 1)
    };
    let (int_word_ptr, int_word_len) = word(0);
    let (string_word_ptr, string_word_len) = word(1);
    let (float_word_ptr, float_word_len) = word(2);
    let (bool_word_ptr, bool_word_len) = word(3);
    let (array_word_ptr, array_word_len) = word(4);
    let (null_word_ptr, null_word_len) = word(5);
    let (resource_word_ptr, resource_word_len) = word(6);
    let value_offset = super::mixed_numeric::CLASS_VALUE_OFFSET;

    // The word a runtime tag contributes to a diagnostic is the SAME wherever the value arrived
    // from — a declared return, a declared parameter — so it is resolved once here and every
    // fatal reads it. An object is why this cannot be a static table: it contributes its CLASS
    // name, which only the runtime class-name table knows.
    wm.add_raw_func(&format!(
        r#"(func $__rt_type_word_for_tag (param $tag i64) (param $lo i64) (result i32) (result i32)
  (local $word_ptr i32) (local $word_len i32) (local $cls_len i64)
  (local.set $word_ptr (i32.const {null_word_ptr}))               ;; tag 8 = null, the default
  (local.set $word_len (i32.const {null_word_len}))
  (if (i64.eq (local.get $tag) (i64.const 1))
    (then (local.set $word_ptr (i32.const {string_word_ptr})) (local.set $word_len (i32.const {string_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 2))
    (then (local.set $word_ptr (i32.const {float_word_ptr})) (local.set $word_len (i32.const {float_word_len}))))
  (if (i64.eqz (local.get $tag))
    (then (local.set $word_ptr (i32.const {int_word_ptr})) (local.set $word_len (i32.const {int_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 3))
    (then (local.set $word_ptr (i32.const {bool_word_ptr})) (local.set $word_len (i32.const {bool_word_len}))))
  (if (i32.or (i64.eq (local.get $tag) (i64.const 4)) (i64.eq (local.get $tag) (i64.const 5)))
    (then (local.set $word_ptr (i32.const {array_word_ptr})) (local.set $word_len (i32.const {array_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 9))
    (then (local.set $word_ptr (i32.const {resource_word_ptr})) (local.set $word_len (i32.const {resource_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 10))                    ;; PHP names a closure by its class
    (then (local.set $word_ptr (i32.const {closure_ptr})) (local.set $word_len (i32.const {closure_len}))))
  (if (i64.eq (local.get $tag) (i64.const 6))                     ;; an object contributes its CLASS name
    (then
      (call $__rt_class_name_by_cid (i64.load (i32.wrap_i64 (local.get $lo))))
      (local.set $cls_len)
      (local.set $word_ptr)
      (local.set $word_len (i32.wrap_i64 (local.get $cls_len)))))
  (local.get $word_ptr) (local.get $word_len))"#
    ));

    // One fatal serves all four targets. Measured: the word a tag contributes is the SAME
    // whatever the declared type — only the target word and the set of ACCEPTED tags differ.
    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_return_type (param $fn_ptr i32) (param $fn_len i32) (param $target_ptr i32) (param $target_len i32) (param $tag i64) (param $lo i64)
  (local $word_ptr i32) (local $word_len i32)
  (call $__rt_type_word_for_tag (local.get $tag) (local.get $lo))
  (local.set $word_len)
  (local.set $word_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {err_prefix_ptr}) (i32.const {err_prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $fn_ptr) (local.get $fn_len)))    ;; "f" or "C::m"
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {err_middle_ptr}) (i32.const {err_middle_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $target_ptr) (local.get $target_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {err_sep_ptr}) (i32.const {err_sep_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $word_ptr) (local.get $word_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {err_suffix_ptr}) (i32.const {err_suffix_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:return-type-fatal-exit
)"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_deprecate_return_float_to_int (param $bits i64)
  (local $ptr i32) (local $len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {float_prefix_ptr}) (i32.const {float_prefix_len}))  ;; "Deprecated: Implicit conversion from float "
  (call $__rt_ftoa (local.get $bits) (i32.add (global.get $__float_scratch) (i32.const 1024)) (i32.const 80) (i32.add (global.get $__float_scratch) (i32.const 2048)) (i32.const 792) (i32.add (global.get $__float_scratch) (i32.const 4096)))  ;; render the float the way PHP prints it
  (local.set $len)                                                ;; ftoa returns (ptr, len): pop the length first
  (local.set $ptr)                                                ;; then the pointer
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (local.get $len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {float_suffix_ptr}) (i32.const {float_suffix_len})))  ;; " to int loses precision\n""#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_deprecate_return_str_to_int (param $ptr i32) (param $len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {str_prefix_ptr}) (i32.const {str_prefix_len}))  ;; ...from float-string "
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (local.get $len))  ;; the ORIGINAL bytes, padding included
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {str_suffix_ptr}) (i32.const {str_suffix_len})))  ;; " to int loses precision\n""#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_deprecate_nan_to_string
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {nan_string_ptr}) (i32.const {nan_string_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_deprecate_nan_to_bool
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {nan_bool_ptr}) (i32.const {nan_bool_len})))"#
    ));

    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_to_int_core (param $cell i32) (result i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $f f64) (local $t f64) (local $cls i32)
  (call $__rt_mixed_unbox (local.get $cell))                      ;; unbox -> stack: tag, lo, hi
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eqz (local.get $tag))                                  ;; tag 0 = int: already PHP's answer
    (then (return (i32.const 1) (local.get $lo))))
  (if (i64.eq (local.get $tag) (i64.const 3))                     ;; tag 3 = bool: stored normalized 0/1
    (then (return (i32.const 1) (local.get $lo))))
  (if (i64.eq (local.get $tag) (i64.const 2))                     ;; tag 2 = float
    (then
      (local.set $f (f64.reinterpret_i64 (local.get $lo)))
      (if (i32.and
            (f64.eq (local.get $f) (local.get $f))                ;; NaN fits nothing
            (i32.and
              (f64.ge (local.get $f) (f64.const -9223372036854775808))   ;; -2^63 is IN range
              (f64.lt (local.get $f) (f64.const 9223372036854775808))))  ;; 2^63 is NOT
        (then
          (local.set $t (f64.trunc (local.get $f)))
          (if (f64.ne (local.get $t) (local.get $f))               ;; a lost fraction only DEPRECATES
            (then (call $__rt_deprecate_return_float_to_int (local.get $lo))))
          (return (i32.const 1) (i64.trunc_f64_s (local.get $t)))))))
  (if (i64.eq (local.get $tag) (i64.const 1))                     ;; tag 1 = string
    (then
      (local.set $cls (call $__rt_str_numeric_class (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi))))
      (if (i32.eq (local.get $cls) (i32.const 1))                 ;; wholly integral, and it fits i64
        (then (return (i32.const 1) (i64.load (i32.add (global.get $__float_scratch) (i32.const {value_offset}))))))
      (if (i32.eq (local.get $cls) (i32.const 2))                 ;; wholly float-shaped (or an i64-overflowing integer text)
        (then
          (local.set $f (f64.load (i32.add (global.get $__float_scratch) (i32.const {value_offset}))))
          (if (i32.and
                (f64.eq (local.get $f) (local.get $f))
                (i32.and
                  (f64.ge (local.get $f) (f64.const -9223372036854775808))
                  (f64.lt (local.get $f) (f64.const 9223372036854775808))))
            (then
              (local.set $t (f64.trunc (local.get $f)))
              (if (f64.ne (local.get $t) (local.get $f))
                (then (call $__rt_deprecate_return_str_to_int (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi)))))
              (return (i32.const 1) (i64.trunc_f64_s (local.get $t)))))))))
  (i32.const 0) (i64.const 0))                                    ;; no conversion exists
"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_return_int (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (result i64)
  (local $ok i32) (local $value i64) (local $tag i64) (local $lo i64) (local $hi i64)
  (call $__rt_mixed_to_int_core (local.get $cell))
  (local.set $value)
  (local.set $ok)
  (if (local.get $ok)
    (then (return (local.get $value))))
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (call $__rt_fail_return_type (local.get $fn_ptr) (local.get $fn_len) (i32.const {int_word_ptr}) (i32.const {int_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:return-coerce-int
"#
    ));
    // The same conversion at a declared PARAMETER, which differs from the return in exactly two
    // places. Measured on php-src 8.5.6 for `substr("abcdefgh", $mixed)`: `null` does NOT raise
    // there — it converts to 0 after a `Deprecated` naming the parameter — and the failure names
    // `Argument #N ($p)` rather than the return value. Everything numeric in between, the two
    // precision deprecations included, is byte-for-byte the same, which is why it comes from the
    // shared core rather than a second copy.
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_arg_int (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (param $param_ptr i32) (param $param_len i32) (param $argno i64) (result i64)
  (local $ok i32) (local $value i64) (local $tag i64) (local $lo i64) (local $hi i64)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eq (local.get $tag) (i64.const 8))                     ;; null still converts, after a deprecation
    (then
      (call $__rt_deprecate_argument_null (local.get $fn_ptr) (local.get $fn_len) (local.get $param_ptr) (local.get $param_len) (local.get $argno) (i32.const {int_word_ptr}) (i32.const {int_word_len}))
      (return (i64.const 0))))
  (call $__rt_mixed_to_int_core (local.get $cell))
  (local.set $value)
  (local.set $ok)
  (if (local.get $ok)
    (then (return (local.get $value))))
  (call $__rt_fail_argument_type (local.get $fn_ptr) (local.get $fn_len) (local.get $param_ptr) (local.get $param_len) (local.get $argno) (i32.const {int_word_ptr}) (i32.const {int_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:argument-coerce-int
"#
    ));
    // A float target takes every numeric tag EXACTLY as `(float)` does — NaN and the infinities
    // included, since none of them loses anything on the way to a float, so there is no notice
    // here at all. Only a string needs gating: a leading-numeric one converts under `(float)`
    // but is a `TypeError` at a return.
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_return_float (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $cls i32)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i32.or (i64.eqz (local.get $tag))
        (i32.or (i64.eq (local.get $tag) (i64.const 2)) (i64.eq (local.get $tag) (i64.const 3))))
    (then (return (call $__rt_mixed_cast_float (local.get $cell)))))  ;; int / float / bool
  (if (i64.eq (local.get $tag) (i64.const 1))
    (then
      (local.set $cls (call $__rt_str_numeric_class (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi))))
      (if (i32.or (i32.eq (local.get $cls) (i32.const 1)) (i32.eq (local.get $cls) (i32.const 2)))  ;; WHOLLY numeric only
        (then (return (call $__rt_mixed_cast_float (local.get $cell)))))))
  (call $__rt_fail_return_type (local.get $fn_ptr) (local.get $fn_len) (i32.const {float_word_ptr}) (i32.const {float_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:return-coerce-tofloat
"#
    ));
    // `string` and `bool` accept every scalar tag with the explicit cast's own value — the two
    // operations only part ways on the tags PHP refuses outright, plus the NaN notice.
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_return_bool (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $f f64)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eq (local.get $tag) (i64.const 2))
    (then
      (local.set $f (f64.reinterpret_i64 (local.get $lo)))
      (if (f64.ne (local.get $f) (local.get $f))                  ;; NaN still converts, after a notice
        (then (call $__rt_deprecate_nan_to_bool)))))
  (if (i32.or (i64.eqz (local.get $tag))
        (i32.or (i64.eq (local.get $tag) (i64.const 1))
          (i32.or (i64.eq (local.get $tag) (i64.const 2)) (i64.eq (local.get $tag) (i64.const 3)))))
    (then (return (call $__rt_mixed_cast_bool (local.get $cell)))))
  (call $__rt_fail_return_type (local.get $fn_ptr) (local.get $fn_len) (i32.const {bool_word_ptr}) (i32.const {bool_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:return-coerce-tobool
"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_return_string (param $cell i32) (param $fn_ptr i32) (param $fn_len i32) (result i32) (result i32)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $f f64)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i64.eq (local.get $tag) (i64.const 2))
    (then
      (local.set $f (f64.reinterpret_i64 (local.get $lo)))
      (if (f64.ne (local.get $f) (local.get $f))
        (then (call $__rt_deprecate_nan_to_string)))))
  (if (i32.or (i64.eqz (local.get $tag))
        (i32.or (i64.eq (local.get $tag) (i64.const 1))
          (i32.or (i64.eq (local.get $tag) (i64.const 2)) (i64.eq (local.get $tag) (i64.const 3)))))
    (then (return (call $__rt_mixed_cast_string (local.get $cell)))))
  (call $__rt_fail_return_type (local.get $fn_ptr) (local.get $fn_len) (i32.const {string_word_ptr}) (i32.const {string_word_len}) (local.get $tag) (local.get $lo))
  unreachable)                                                    ;; elephc-trap:post-noreturn:return-coerce-tostring
"#
    ));
    // `count()` on a boxed value. Its own word table because a boolean is named by VALUE here,
    // and its own message because an internal function's `TypeError` carries no location.
    wm.add_raw_func(&format!(
        r#"(func $__rt_mixed_count (param $cell i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $word_ptr i32) (local $word_len i32) (local $cls_len i64)
  (call $__rt_mixed_unbox (local.get $cell))
  (local.set $hi)
  (local.set $lo)
  (local.set $tag)
  (if (i32.or (i64.eq (local.get $tag) (i64.const 4)) (i64.eq (local.get $tag) (i64.const 5)))
    (then (return (i64.load (i32.wrap_i64 (local.get $lo))))))    ;; element count @ +0
  (if (i64.eq (local.get $tag) (i64.const 6))
    (then
      (call $__rt_fail (i32.const 9))                             ;; a Countable object is not lowered on this target
      unreachable))                                               ;; elephc-trap:post-noreturn:count-object
  (local.set $word_ptr (i32.const {null_word_ptr}))               ;; tag 8 = null, the default
  (local.set $word_len (i32.const {null_word_len}))
  (if (i64.eqz (local.get $tag))
    (then (local.set $word_ptr (i32.const {int_word_ptr})) (local.set $word_len (i32.const {int_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 1))
    (then (local.set $word_ptr (i32.const {string_word_ptr})) (local.set $word_len (i32.const {string_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 2))
    (then (local.set $word_ptr (i32.const {float_word_ptr})) (local.set $word_len (i32.const {float_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 3))                     ;; a boolean is named by its VALUE
    (then
      (local.set $word_ptr (i32.const {false_ptr}))
      (local.set $word_len (i32.const {false_len}))
      (if (i64.ne (local.get $lo) (i64.const 0))
        (then (local.set $word_ptr (i32.const {true_ptr})) (local.set $word_len (i32.const {true_len}))))))
  (if (i64.eq (local.get $tag) (i64.const 9))
    (then (local.set $word_ptr (i32.const {resource_word_ptr})) (local.set $word_len (i32.const {resource_word_len}))))
  (if (i64.eq (local.get $tag) (i64.const 10))
    (then (local.set $word_ptr (i32.const {closure_ptr})) (local.set $word_len (i32.const {closure_len}))))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {count_prefix_ptr}) (i32.const {count_prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $word_ptr) (local.get $word_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {count_suffix_ptr}) (i32.const {count_suffix_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable)                                                    ;; elephc-trap:post-noreturn:count-type-fatal-exit
"#
    ));
    // `$s[$i]`: PHP counts a negative index from the END, and answers the EMPTY string with a
    // warning for anything still outside. The warning names the index AS WRITTEN.
    wm.add_raw_func(&format!(
        r#"(func $__rt_str_char_at (param $ptr i32) (param $len i64) (param $idx i64) (result i32) (result i64)
  (local $i i64) (local $tp i32) (local $tl i32)
  (local.set $i (local.get $idx))
  (if (i64.lt_s (local.get $i) (i64.const 0))                     ;; a negative index counts from the end
    (then (local.set $i (i64.add (local.get $len) (local.get $i)))))
  (if (i32.or (i64.lt_s (local.get $i) (i64.const 0)) (i64.ge_s (local.get $i) (local.get $len)))
    (then
      (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {uninit_offset_ptr}) (i32.const {uninit_offset_len}))
      (call $__rt_itoa (local.get $idx) (i32.add (global.get $__float_scratch) (i32.const 9344)))
      (local.set $tl)
      (local.set $tp)
      (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $tp) (local.get $tl))
      (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {newline_ptr}) (i32.const {newline_len}))
      (return (call $__rt_str_persist (local.get $ptr) (i64.const 0)))))  ;; PHP answers ""
  (call $__rt_str_persist                                         ;; one byte, owned by the caller
    (i32.add (local.get $ptr) (i32.wrap_i64 (local.get $i)))
    (i64.const 1)))
"#
    ));
}

/// Emits the fatal path used when an object has no matching method dispatch arm.
///
/// The runtime class-name table supplies the concrete class name while the
/// instruction's interned method-name bytes complete PHP's undefined-method text.
fn emit_undefined_method_failure_runtime(wm: &mut WatModule, offsets: &[(u32, u32)]) {
    debug_assert_eq!(offsets.len(), 3);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (separator_ptr, separator_len) = offsets[1];
    let (suffix_ptr, suffix_len) = offsets[2];
    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_undefined_method (param $cid i64) (param $method_ptr i32) (param $method_len i32)
  (local $class_ptr i32) (local $class_len i64)
  (call $__rt_class_name_by_cid (local.get $cid))
  (local.set $class_len)
  (local.set $class_ptr)
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $class_ptr) (i32.wrap_i64 (local.get $class_len))))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {separator_ptr}) (i32.const {separator_len})))
  (drop (call $__rt_wasi_write_all (i32.const 2) (local.get $method_ptr) (local.get $method_len)))
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:undefined-method-fatal-exit
)"#
    ));
}

/// Emits PHP's non-fatal warning for a missing integer array index.
///
/// The key is formatted through the shared signed `__rt_itoa` helper, including
/// `i64::MIN`, and every stderr fragment uses the checked WASI write path. The
/// helper returns normally so the caller can continue with the already-produced
/// null value. A no-argument companion emits the exact offset-on-null warning.
fn emit_undefined_array_key_warning_runtime(
    wm: &mut WatModule,
    offsets: &[(u32, u32)],
) {
    debug_assert_eq!(offsets.len(), 17);
    let (prefix_ptr, prefix_len) = offsets[0];
    let (quote_ptr, quote_len) = offsets[1];
    let (suffix_ptr, suffix_len) = offsets[2];
    let (chr_range_ptr, chr_range_len) = offsets[12];
    let (ord_length_ptr, ord_length_len) = offsets[13];
    let (array_to_string_ptr, array_to_string_len) = offsets[14];
    let (object_string_prefix_ptr, object_string_prefix_len) = offsets[15];
    let (object_string_suffix_ptr, object_string_suffix_len) = offsets[16];
    let (float_prefix_ptr, float_prefix_len) = offsets[5];
    let (float_suffix_ptr, float_suffix_len) = offsets[6];
    let (non_numeric_ptr, non_numeric_len) = offsets[7];
    let (operand_ptr, operand_len) = offsets[8];
    let (object_prefix_ptr, object_prefix_len) = offsets[9];
    let (object_int_ptr, object_int_len) = offsets[10];
    let (object_float_ptr, object_float_len) = offsets[11];
    let (offset_on_null_ptr, offset_on_null_len) =
        if crate::codegen_support::runtime::array_offset_on_null_warning()
            == crate::ir::ARRAY_OFFSET_ON_NULL_WARNING_PHP82
        {
            offsets[3]
        } else {
            offsets[4]
        };
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_undefined_array_key_int (param $key i64)
  (local $key_ptr i32) (local $key_len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len}))
  (call $__rt_itoa (local.get $key) (global.get $__float_scratch))
  (local.set $key_len)
  (local.set $key_ptr)
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $key_ptr) (local.get $key_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_undefined_array_key_str (param $key_ptr i32) (param $key_len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {prefix_ptr}) (i32.const {prefix_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {quote_ptr}) (i32.const {quote_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $key_ptr) (local.get $key_len))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {quote_ptr}) (i32.const {quote_len}))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {suffix_ptr}) (i32.const {suffix_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_fail_object_to_string (param $cid i64)
  (local $ptr i32) (local $len i64)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_string_prefix_ptr}) (i32.const {object_string_prefix_len}))
  (call $__rt_class_name_by_cid (local.get $cid))                 ;; resolve the class name -> (ptr, len)
  (local.set $len)
  (local.set $ptr)
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (i32.wrap_i64 (local.get $len)))
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_string_suffix_ptr}) (i32.const {object_string_suffix_len}))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:object-to-string-fatal
)"#
    ));
    wm.add_raw_func(
        r#"(func $__rt_echo_array_word
  ;; The five bytes of "Array" are written into the float scratch rather than carried as a
  ;; data segment, so this stays independent of the module's static-data layout.
  (i32.store8 (global.get $__float_scratch) (i32.const 65))
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const 1)) (i32.const 114))
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const 2)) (i32.const 114))
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const 3)) (i32.const 97))
  (i32.store8 (i32.add (global.get $__float_scratch) (i32.const 4)) (i32.const 121))
  (call $__rt_echo_str (global.get $__float_scratch) (i64.const 5)))"#,
    );
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_array_to_string
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {array_to_string_ptr}) (i32.const {array_to_string_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_array_offset_on_null
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {offset_on_null_ptr}) (i32.const {offset_on_null_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_float_not_representable (param $bits i64)
  (local $ptr i32) (local $len i32)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {float_prefix_ptr}) (i32.const {float_prefix_len}))  ;; "Warning: The float "
  (call $__rt_ftoa (local.get $bits) (i32.add (global.get $__float_scratch) (i32.const 1024)) (i32.const 80) (i32.add (global.get $__float_scratch) (i32.const 2048)) (i32.const 792) (i32.add (global.get $__float_scratch) (i32.const 4096)))  ;; render the offending float exactly as PHP prints it
  (local.set $len)                                                ;; ftoa returns (ptr, len): pop the length first
  (local.set $ptr)                                                ;; then the pointer
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (local.get $len))  ;; the float text itself
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {float_suffix_ptr}) (i32.const {float_suffix_len})))  ;; " is not representable as an int, cast occurred\n""#
    ));
    // The class name is looked up from the runtime class id, so one helper per target type
    // covers every class rather than needing a per-class message.
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_object_to_int (param $cid i64)
  (local $ptr i32) (local $len i64)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_prefix_ptr}) (i32.const {object_prefix_len}))  ;; "Warning: Object of class "
  (call $__rt_class_name_by_cid (local.get $cid))                 ;; resolve the class name -> (ptr, len)
  (local.set $len)                                                ;; pop the name length
  (local.set $ptr)                                                ;; pop the name pointer
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (i32.wrap_i64 (local.get $len)))  ;; the class name itself
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_int_ptr}) (i32.const {object_int_len})))  ;; " could not be converted to int\n""#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_object_to_float (param $cid i64)
  (local $ptr i32) (local $len i64)
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_prefix_ptr}) (i32.const {object_prefix_len}))  ;; "Warning: Object of class "
  (call $__rt_class_name_by_cid (local.get $cid))                 ;; resolve the class name -> (ptr, len)
  (local.set $len)                                                ;; pop the name length
  (local.set $ptr)                                                ;; pop the name pointer
  (call $__rt_wasi_write_or_fail (i32.const 2) (local.get $ptr) (i32.wrap_i64 (local.get $len)))  ;; the class name itself
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {object_float_ptr}) (i32.const {object_float_len})))  ;; " could not be converted to float\n""#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_warn_non_numeric_value
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {non_numeric_ptr}) (i32.const {non_numeric_len})))"#
    ));
    wm.add_raw_func(&format!(
        r#"(func $__rt_fatal_unsupported_operand
  (drop (call $__rt_wasi_write_all (i32.const 2) (i32.const {operand_ptr}) (i32.const {operand_len})))
  (call $wasi_proc_exit (i32.const 255))
  unreachable ;; elephc-trap:post-noreturn:unsupported-operand-fatal-exit
)"#
    ));
    if matches!(
        crate::codegen_support::compile_php_version(),
        crate::web_prelude::PhpVersion::Php85
    ) {
        // Registered here, not in `emit_float_runtime`: the diagnosing conversion
        // depends on the warning helper above, which only command modules carry.
        wm.add_raw_func(super::float::RT_FLOAT_TO_INT_WARN);
        // PHP 8.5 alone deprecates a `chr()` argument outside a byte and an `ord()` argument
        // that is not exactly one byte. Both still ANSWER — the value is unchanged, only the
        // diagnostic is new — so earlier profiles get the same result with no message.
        wm.add_raw_func(&format!(
            r#"(func $__rt_deprecated_chr_range
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {chr_range_ptr}) (i32.const {chr_range_len})))"#
        ));
        wm.add_raw_func(&format!(
            r#"(func $__rt_deprecated_ord_length
  (call $__rt_wasi_write_or_fail (i32.const 2) (i32.const {ord_length_ptr}) (i32.const {ord_length_len})))"#
        ));
    }
    super::mixed_numeric::emit_mixed_numeric_runtime(wm);
}

/// Repeatedly invokes WASI `fd_write` until every requested byte is written.
///
/// Returns the first host errno. A zero-progress write or an impossible
/// `nwritten > remaining` response returns WASI `ERRNO_IO` (29), preventing an
/// infinite loop or pointer underflow. The single iovec and `nwritten` cell use
/// the reserved low-memory scratch region.
const RT_WASI_WRITE_ALL: &str =
    r#"(func $__rt_wasi_write_all (param $fd i32) (param $ptr i32) (param $len i32) (result i32)
  (local $remaining i32) (local $cursor i32) (local $errno i32) (local $written i32)
  (local.set $remaining (local.get $len))                         ;; bytes still to write
  (local.set $cursor (local.get $ptr))                            ;; next byte address
  (block $done
    (loop $write
      (br_if $done (i32.eqz (local.get $remaining)))              ;; all bytes written
      (i32.store (i32.const 0) (local.get $cursor))               ;; iovec.buf_ptr
      (i32.store (i32.const 4) (local.get $remaining))            ;; iovec.buf_len
      (local.set $errno
        (call $wasi_fd_write (local.get $fd) (i32.const 0) (i32.const 1) (i32.const 8))) ;; host write
      (if (i32.ne (local.get $errno) (i32.const 0))
        (then (return (local.get $errno))))                       ;; propagate host errno
      (local.set $written (i32.load (i32.const 8)))               ;; bytes accepted by host
      (if (i32.or
            (i32.eqz (local.get $written))
            (i32.gt_u (local.get $written) (local.get $remaining)))
        (then (return (i32.const 29))))                           ;; ERRNO_IO on no/invalid progress
      (local.set $cursor (i32.add (local.get $cursor) (local.get $written))) ;; advance source
      (local.set $remaining (i32.sub (local.get $remaining) (local.get $written))) ;; shrink tail
      (br $write)))
  (i32.const 0))                                                  ;; success"#;

/// Writes every requested byte or converts a WASI host error into the command
/// runtime's deterministic fatal diagnostic and exit status.
///
/// `__rt_fail` deliberately calls `__rt_wasi_write_all` directly for its
/// best-effort stderr diagnostic, avoiding recursion if stderr itself fails.
const RT_WASI_WRITE_OR_FAIL: &str =
    r#"(func $__rt_wasi_write_or_fail (param $fd i32) (param $ptr i32) (param $len i32)
  (if (i32.ne
        (call $__rt_wasi_write_all (local.get $fd) (local.get $ptr) (local.get $len))
        (i32.const 0))
    (then
      (call $__rt_fail (i32.const 5))
      unreachable))) ;; elephc-trap:post-noreturn:wasi-write-failure
"#;

/// `__rt_argc`: returns PHP's `$argc` (the process argument count) via WASI
/// `args_sizes_get`, which writes the count to the number-buffer scratch region.
const RT_ARGC: &str = r#"(func $__rt_argc (result i64)
  (local $errno i32)
  (local.set $errno (call $wasi_args_sizes_get (i32.const 16) (i32.const 20))) ;; argc@16, argv_buf_size@20
  (if (i32.ne (local.get $errno) (i32.const 0))
    (then
      (call $__rt_fail (i32.const 5))
      unreachable))                                               ;; elephc-trap:post-noreturn:argc-sizes-failure args_sizes_get failed
  (i64.extend_i32_u (i32.load (i32.const 16))))                    ;; return argc as i64"#;

/// `__rt_strlen_c`: byte length of a NUL-terminated C string (used to measure the
/// WASI argv entries before copying them into PHP strings).
const RT_STRLEN_C: &str = r#"(func $__rt_strlen_c (param $p i32) (result i32)
  (local $n i32)
  (local.set $n (i32.const 0))                               ;; n = 0
  (block $end (loop $scan
    (br_if $end (i32.eqz (i32.load8_u (i32.add (local.get $p) (local.get $n)))))  ;; stop at the NUL terminator
    (local.set $n (i32.add (local.get $n) (i32.const 1)))    ;; n++
    (br $scan)))                                             ;; continue scanning
  (local.get $n))                                            ;; return byte count"#;

/// `__rt_argv`: builds PHP's `$argv` as an indexed string array via WASI
/// `args_sizes_get` + `args_get`. Temporary heap buffers hold the WASI pointer
/// array and argument byte buffer; each argument is copied (persisted) into the
/// array via `__rt_array_push_str`, after which the temporaries are freed.
const RT_ARGV: &str = r#"(func $__rt_argv (result i32)
  (local $argc i32)
  (local $bufsize i32)
  (local $ptrs i32)
  (local $buf i32)
  (local $arr i32)
  (local $i i32)
  (local $argp i32)
  (local $len i32)
  (local $errno i32)
  (local.set $errno (call $wasi_args_sizes_get (i32.const 16) (i32.const 20))) ;; query argc and byte size
  (if (i32.ne (local.get $errno) (i32.const 0))
    (then
      (call $__rt_fail (i32.const 5))
      unreachable))                                                 ;; elephc-trap:post-noreturn:argv-sizes-failure args_sizes_get failed
  (local.set $argc (i32.load (i32.const 16)))                        ;; load argc from scratch
  (local.set $bufsize (i32.load (i32.const 20)))                     ;; load argv byte-buffer size
  (if (i32.gt_u (local.get $argc) (i32.const 1073741823))
    (then
      (call $__rt_fail (i32.const 5))
      unreachable))                                                 ;; elephc-trap:post-noreturn:argv-count-overflow argc * 4 must not wrap wasm32
  (local.set $ptrs (call $__rt_heap_alloc (i32.mul (local.get $argc) (i32.const 4))))  ;; argc i32 pointers
  (local.set $buf (call $__rt_heap_alloc (local.get $bufsize)))      ;; argv byte buffer
  (local.set $errno (call $wasi_args_get (local.get $ptrs) (local.get $buf))) ;; fill pointer array + buffer
  (if (i32.ne (local.get $errno) (i32.const 0))
    (then
      (call $__rt_heap_free (local.get $ptrs))
      (call $__rt_heap_free (local.get $buf))
      (call $__rt_fail (i32.const 5))
      unreachable))                                                 ;; elephc-trap:post-noreturn:argv-get-failure args_get failed after balanced cleanup
  (local.set $arr (call $__rt_array_new (i64.extend_i32_u (local.get $argc)) (i64.const 16)))  ;; string array
  (local.set $i (i32.const 0))                                       ;; i = 0
  (block $end (loop $loop
    (br_if $end (i32.ge_u (local.get $i) (local.get $argc)))         ;; exit loop when all args processed
    (local.set $argp (i32.load (i32.add (local.get $ptrs) (i32.mul (local.get $i) (i32.const 4)))))  ;; argv[i] (C string)
    (local.set $len (call $__rt_strlen_c (local.get $argp)))         ;; its byte length
    (local.set $arr (call $__rt_array_push_str (local.get $arr) (local.get $argp) (i64.extend_i32_u (local.get $len))))  ;; append a persisted copy
    (local.set $i (i32.add (local.get $i) (i32.const 1)))            ;; i++
    (br $loop)))                                                     ;; next arg
  (call $__rt_heap_free (local.get $ptrs))                          ;; temporaries no longer needed (args were copied)
  (call $__rt_heap_free (local.get $buf))                            ;; free the argument byte buffer
  (local.get $arr))                                                  ;; return the argv array"#;

/// `__rt_mixed_write_stdout`: echoes a boxed Mixed value by dispatching on its tag:
/// int (0) via `__rt_echo_i64`, float (2) via `__rt_echo_f64` (`%.14G`), string (1)
/// via `__rt_echo_str`, bool (3) via `__rt_echo_bool`; null (8) and non-scalar tags
/// print nothing (PHP semantics).
const RT_MIXED_WRITE_STDOUT: &str = r#"(func $__rt_mixed_write_stdout (param $ptr i32)
  (local $tag i64)
  (local $sptr i32)
  (local $len i32)
  (if (i32.eqz (local.get $ptr))
    (then (return)))                                                ;; null pointer -> nothing
  (local.set $tag (i64.load (local.get $ptr)))                      ;; tag @ +0
  (if (i64.eqz (local.get $tag))                                    ;; tag 0 = int
    (then
      (call $__rt_echo_i64 (i64.load (i32.add (local.get $ptr) (i32.const 8)))) ;; echo int payload (lo @ +8)
      (return)))                                                     ;; done
  (if (i64.eq (local.get $tag) (i64.const 1))                       ;; tag 1 = string
    (then
      (call $__rt_echo_str
        (i32.wrap_i64 (i64.load (i32.add (local.get $ptr) (i32.const 8))))
        (i64.load (i32.add (local.get $ptr) (i32.const 16))))        ;; echo string (ptr, len)
      (return)))                                                     ;; done
  (if (i64.eq (local.get $tag) (i64.const 2))                       ;; tag 2 = float
    (then
      (call $__rt_echo_f64 (f64.load (i32.add (local.get $ptr) (i32.const 8)))) ;; %.14G text via __rt_ftoa + fd_write
      (return)))                                                     ;; done
  (if (i64.eq (local.get $tag) (i64.const 3))                       ;; tag 3 = bool
    (then
      (call $__rt_echo_bool (i64.load (i32.add (local.get $ptr) (i32.const 8)))) ;; echo bool payload
      (return)))
  (if (i32.or (i64.eq (local.get $tag) (i64.const 4)) (i64.eq (local.get $tag) (i64.const 5)))
    (then
      ;; PHP prints the literal text "Array" and warns; the cast helper owns both, and its
      ;; persisted result is released here because echoing keeps nothing.
      (call $__rt_mixed_cast_string (local.get $ptr))
      (local.set $len)                                               ;; pop (ptr, len)
      (local.set $sptr)
      (call $__rt_echo_str (local.get $sptr) (i64.extend_i32_u (local.get $len)))  ;; "Array"
      (call $__rt_heap_free_safe (local.get $sptr))                  ;; the echo owns nothing
      (return))))                                                    ;; done
"#;

/// `__rt_concat`: allocates an owned string and copies `a` then `b` into it.
///
/// Length addition and wasm32 narrowing are checked in i64 before allocation.
/// The returned block is stamped with runtime kind 1 so ordinary ownership
/// release frees intermediate concatenations. This avoids shared-buffer overlap
/// across recursion, calls, and strings larger than 64 KiB.
const RT_CONCAT: &str = r#"(func $__rt_concat (param $aptr i32) (param $alen i64) (param $bptr i32) (param $blen i64) (result i32) (result i64)
  (local $total i64) (local $al i32) (local $bl i32) (local $result i32)
  (if (i32.or
        (i64.lt_s (local.get $alen) (i64.const 0))
        (i64.lt_s (local.get $blen) (i64.const 0)))
    (then
      (call $__rt_oom)
      unreachable))                                          ;; elephc-trap:deterministic-oom:concat-negative-length malformed negative length
  (local.set $total (i64.add (local.get $alen) (local.get $blen))) ;; widened total length
  (if (i32.or
        (i64.lt_u (local.get $total) (local.get $alen))
        (i64.gt_u (local.get $total) (i64.const 4294900736)))
    (then
      (call $__rt_oom)
      unreachable))                                          ;; elephc-trap:deterministic-oom:concat-length-overflow overflow or unaddressable wasm32 length
  (local.set $al (i32.wrap_i64 (local.get $alen)))           ;; safe after total-length bound
  (local.set $bl (i32.wrap_i64 (local.get $blen)))           ;; safe after total-length bound
  (local.set $result (call $__rt_heap_alloc (i32.wrap_i64 (local.get $total)))) ;; owned result bytes
  (i64.store (i32.sub (local.get $result) (i32.const 8)) (i64.const 1)) ;; runtime kind = string
  (memory.copy (local.get $result) (local.get $aptr) (local.get $al)) ;; copy lhs bytes
  (memory.copy
    (i32.add (local.get $result) (local.get $al))
    (local.get $bptr)
    (local.get $bl))                                          ;; append rhs bytes
  (local.get $result)                                         ;; owned result pointer
  (local.get $total))                                         ;; result length"#;

/// `__rt_echo_bool`: PHP `echo` of a boolean writes "1" for true and nothing for
/// false. The value is the i64 boolean (0 or 1).
const RT_ECHO_BOOL: &str = r#"(func $__rt_echo_bool (param $v i64)
  (if (i64.ne (local.get $v) (i64.const 0))
    (then
      (i32.store8 (i32.const 16) (i32.const 49))            ;; '1' into the number buffer
      (call $__rt_wasi_write_or_fail (i32.const 1) (i32.const 16) (i32.const 1))))) ;; write "1""#;

/// `__rt_echo_str`: writes a string (a linear-memory pointer + byte length) to
/// stdout via `fd_write`. The length is an i64 (PHP int) wrapped to the i32 the
/// iovec field requires.
const RT_ECHO_STR: &str = r#"(func $__rt_echo_str (param $ptr i32) (param $len i64)
  (if (i64.gt_u (local.get $len) (i64.const 4294967295))
    (then
      (call $__rt_fail (i32.const 5))
      unreachable))                                          ;; elephc-trap:post-noreturn:echo-string-length-overflow wasm32 cannot address a larger byte range
  (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (i32.wrap_i64 (local.get $len)))) ;; write to stdout"#;

/// `__rt_echo_i64`: writes a signed 64-bit integer to stdout as decimal text.
///
/// Formats the value back-to-front into the scratch number buffer [16, 64), then
/// points the iovec at the written bytes and calls `fd_write(1, ...)`. The
/// magnitude is taken as unsigned (`0 - v`), which wraps correctly for `i64::MIN`
/// so `div_u`/`rem_u` produce its true digits.
const RT_ECHO_I64: &str = r#"(func $__rt_echo_i64 (param $v i64)
  (local $ptr i32)   ;; back-to-front write cursor into the number buffer
  (local $neg i32)   ;; 1 if the value is negative
  (local $u i64)     ;; magnitude (unsigned)
  (local $len i32)   ;; number of bytes written
  (local.set $ptr (i32.const 64))                              ;; buffer end (exclusive)
  (if (i64.eqz (local.get $v))
    (then
      (local.set $ptr (i32.sub (local.get $ptr) (i32.const 1))) ;; back up one byte for '0'
      (i32.store8 (local.get $ptr) (i32.const 48)))            ;; '0'
    (else
      (local.set $neg (i64.lt_s (local.get $v) (i64.const 0))) ;; sign
      (if (local.get $neg)
        (then (local.set $u (i64.sub (i64.const 0) (local.get $v)))) ;; magnitude (MIN wraps -> correct unsigned)
        (else (local.set $u (local.get $v))))                  ;; positive: magnitude = v
      (block $done
        (loop $digit
          (br_if $done (i64.eqz (local.get $u)))               ;; stop when no digits left
          (local.set $ptr (i32.sub (local.get $ptr) (i32.const 1))) ;; back up one byte for digit
          (i32.store8 (local.get $ptr)
            (i32.add (i32.const 48)
              (i32.wrap_i64 (i64.rem_u (local.get $u) (i64.const 10))))) ;; '0' + (u % 10)
          (local.set $u (i64.div_u (local.get $u) (i64.const 10)))      ;; u /= 10
          (br $digit)))                                        ;; next digit
      (if (local.get $neg)
        (then
          (local.set $ptr (i32.sub (local.get $ptr) (i32.const 1))) ;; back up one byte for '-'
          (i32.store8 (local.get $ptr) (i32.const 45))))))     ;; '-'
  (local.set $len (i32.sub (i32.const 64) (local.get $ptr)))   ;; byte count
  (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (local.get $len))) ;; write to stdout"#;

/// `__rt_echo_f64`: writes a PHP float to stdout as `%.14G` text. The float arrives
/// as a wasm `f64`; its bits are reinterpreted to an `i64` for `__rt_ftoa`, which
/// renders into the float-scratch output region (scratch+4096) and returns
/// `(ptr, len)`. The iovec at [0, 16) is then pointed at those bytes and `fd_write`
/// flushes them to stdout. Mirrors `__rt_echo_str` once the text is materialized.
const RT_ECHO_F64: &str = r#"(func $__rt_echo_f64 (param $v f64)
  (local $bits i64)                                         ;; f64 bits handed to __rt_ftoa
  (local $ptr i32)                                          ;; formatted text pointer (from __rt_ftoa)
  (local $len i32)                                          ;; formatted text length (from __rt_ftoa)
  (local.set $bits (i64.reinterpret_f64 (local.get $v)))    ;; f64 value -> raw bits for __rt_ftoa
  (call $__rt_ftoa (local.get $bits) (i32.add (global.get $__float_scratch) (i32.const 1024)) (i32.const 80) (i32.add (global.get $__float_scratch) (i32.const 2048)) (i32.const 792) (i32.add (global.get $__float_scratch) (i32.const 4096))) ;; format into scratch+4096 -> (ptr,len)
  (local.set $len)                                          ;; pop ftoa length (result 1, on top)
  (local.set $ptr)                                          ;; pop ftoa pointer (result 0)
  (call $__rt_wasi_write_or_fail (i32.const 1) (local.get $ptr) (local.get $len))) ;; write to stdout"#;

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Runtime regression tests for heap-backed string concatenation.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Modules are import-free reactors containing the common runtime and heap.
    //! - Tests validate the bytes with `wasmparser` and execute under Wasmer when
    //!   available, including strings larger than the former 64 KiB buffer.

    use super::{
        emit_common_runtime, ERR_DIV_ZERO, ERR_INTDIV_OVERFLOW, ERR_MOD_ZERO, ERR_NEG_SHIFT,
        ERR_EXPLODE_EMPTY_SEP, ERR_STR_PAD_EMPTY, ERR_STR_REPEAT_NEGATIVE, ERR_STR_SPLIT_LENGTH,
        RT_ARGV, RT_ECHO_BOOL, RT_ECHO_F64, RT_ECHO_I64, RT_ECHO_STR, RT_WASI_WRITE_OR_FAIL,
    };
    use super::super::heap::emit_heap_runtime;
    use super::super::objects::CATCHABLE_RUNTIME_ERRORS;
    use super::super::wat::{DataSegment, WatModule};
    use std::sync::atomic::{AtomicU32, Ordering};

    static TMP_SEQ: AtomicU32 = AtomicU32::new(0);

    /// Verifies a raised runtime error and its uncaught fatal name the same class and message.
    ///
    /// These two halves live apart on purpose: the raise site builds the object a `catch` will
    /// match, while `__rt_fail` owns the text `main` prints when no clause matched. Nothing in
    /// the emitter forces them to agree, so changing one and not the other would let a program
    /// catch a `DivisionByZeroError` yet report an `ArithmeticError` when it does not.
    #[test]
    fn raised_runtime_errors_agree_with_their_uncaught_diagnostics() {
        let fatals = [
            (1, ERR_DIV_ZERO),
            (2, ERR_MOD_ZERO),
            (3, ERR_NEG_SHIFT),
            (4, ERR_INTDIV_OVERFLOW),
            (11, ERR_STR_REPEAT_NEGATIVE),
            (12, ERR_STR_PAD_EMPTY),
            (13, ERR_EXPLODE_EMPTY_SEP),
            (14, ERR_STR_SPLIT_LENGTH),
        ];
        for (code, class_name, message) in CATCHABLE_RUNTIME_ERRORS {
            let (_, fatal) = fatals
                .iter()
                .find(|(fatal_code, _)| *fatal_code == code)
                .unwrap_or_else(|| panic!("failure code {code} has no registered fatal message"));
            let fatal = String::from_utf8_lossy(fatal).to_string();
            assert_eq!(
                fatal,
                format!("PHP Fatal error: Uncaught {class_name}: {message}\n"),
                "failure code {code} raises a different class or message than it reports"
            );
        }
    }

    /// Verifies every PHP stdout helper converts a non-zero WASI errno into the
    /// shared fatal path and that `$argv` rejects pointer-table multiplication
    /// overflow before allocating.
    #[test]
    fn command_runtime_propagates_write_errors_and_guards_argv_size() {
        for echo in [RT_ECHO_BOOL, RT_ECHO_STR, RT_ECHO_I64, RT_ECHO_F64] {
            assert!(
                echo.contains("call $__rt_wasi_write_or_fail"),
                "echo helper bypasses the checked WASI write path:\n{echo}"
            );
            assert!(
                !echo.contains("drop (call $__rt_wasi_write_all"),
                "echo helper still discards the WASI errno:\n{echo}"
            );
        }
        assert!(RT_WASI_WRITE_OR_FAIL.contains("call $__rt_fail (i32.const 5)"));
        assert!(
            RT_ARGV.contains("i32.gt_u (local.get $argc) (i32.const 1073741823)"),
            "$argv must reject argc * 4 overflow before heap allocation"
        );
    }

    /// Returns whether the Wasmer CLI is available for runtime assertions.
    fn wasmer_available() -> bool {
        std::process::Command::new("wasmer")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Builds, validates, and invokes an import-free runtime driver.
    fn run_concat_driver(
        pages: u32,
        heap_base: u32,
        segments: &[(u32, Vec<u8>)],
        driver: &str,
    ) -> Option<String> {
        let mut module = WatModule::new();
        module.set_memory(pages, Some("memory"));
        emit_common_runtime(&mut module);
        emit_heap_runtime(&mut module, heap_base, pages * 65536);
        for (offset, bytes) in segments {
            module.add_data(DataSegment {
                offset: *offset,
                bytes: bytes.clone(),
            });
        }
        module.add_raw_func(driver);
        let wat = module.render();
        let bytes =
            ::wat::parse_str(&wat).unwrap_or_else(|error| panic!("invalid WAT: {error}\n{wat}"));
        wasmparser::validate(&bytes)
            .unwrap_or_else(|error| panic!("invalid WASM: {error}\n{wat}"));
        if !wasmer_available() {
            return None;
        }
        let sequence = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "elephc_wasm_concat_{}_{}",
            std::process::id(),
            sequence
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("m.wasm");
        std::fs::write(&path, bytes).expect("write wasm");
        let output = std::process::Command::new("wasmer")
            .arg("run")
            .arg("--invoke")
            .arg("t")
            .arg(&path)
            .output()
            .expect("run wasmer");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            output.status.success(),
            "concat driver failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Verifies a concatenation larger than 64 KiB grows the heap and preserves bytes.
    #[test]
    fn concat_grows_beyond_the_legacy_buffer() {
        let driver = r#"(func $t (export "t") (result i64)
  (local $ptr i32) (local $len i64)
  (call $__rt_concat (i32.const 90000) (i64.const 70000) (i32.const 160000) (i64.const 1))
  (local.set $len)
  (local.set $ptr)
  (i64.add
    (i64.mul (local.get $len) (i64.const 100000))
    (i64.add
      (i64.mul (i64.extend_i32_u (i32.load8_u (local.get $ptr))) (i64.const 100))
      (i64.extend_i32_u (i32.load8_u (i32.add (local.get $ptr) (i32.const 70000)))))))"#;
        let segments = [
            (90000, vec![b'a'; 70000]),
            (160000, vec![b'Z']),
        ];
        if let Some(output) = run_concat_driver(4, 170000, &segments, driver) {
            assert_eq!(output, "7000109790");
        }
    }

    /// Verifies a later concatenation cannot overwrite an earlier live result.
    #[test]
    fn concurrent_live_concat_results_do_not_alias() {
        let driver = r#"(func $t (export "t") (result i64)
  (local $first i32) (local $second i32)
  (call $__rt_concat (i32.const 90000) (i64.const 2) (i32.const 90002) (i64.const 2))
  drop
  (local.set $first)
  (call $__rt_concat (i32.const 90004) (i64.const 2) (i32.const 90006) (i64.const 2))
  drop
  (local.set $second)
  (drop (local.get $second))
  (i64.or
    (i64.shl (i64.extend_i32_u (i32.load8_u (local.get $first))) (i64.const 24))
    (i64.or
      (i64.shl (i64.extend_i32_u (i32.load8_u (i32.add (local.get $first) (i32.const 1)))) (i64.const 16))
      (i64.or
        (i64.shl (i64.extend_i32_u (i32.load8_u (i32.add (local.get $first) (i32.const 2)))) (i64.const 8))
        (i64.extend_i32_u (i32.load8_u (i32.add (local.get $first) (i32.const 3))))))))"#;
        let segments = [
            (90000, b"AB".to_vec()),
            (90002, b"CD".to_vec()),
            (90004, b"xy".to_vec()),
            (90006, b"zz".to_vec()),
        ];
        if let Some(output) = run_concat_driver(2, 100000, &segments, driver) {
            assert_eq!(output, "1094861636");
        }
    }
}
