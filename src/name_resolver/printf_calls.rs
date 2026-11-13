//! Purpose:
//! Routes `sprintf`-family calls the native formatter cannot reproduce exactly to the
//! elephc-PHP implementation in `crate::printf_prelude`.
//!
//! Called from:
//! - `crate::name_resolver::expressions::resolve_expr` for every resolved builtin call.
//!
//! Key details:
//! - The native `__rt_sprintf` path stays for literal formats whose every conversion is a
//!   plain `%d`/`%x`/`%X`/`%o` (flags and width), `%f` (flags other than `0`, width,
//!   precision) or bare `%s`/`%c`, where libc's output equals PHP's and each argument is converted at compile
//!   time; everything else (dynamic formats, `%1$s`, `*`, `'c` padding, `b`/`u`/`e`/`g`/`F`,
//!   the `l` modifier, spread or named arguments, too few arguments) goes to the prelude.
//! - `vsprintf`, `vprintf`, `fprintf` and `vfprintf` always use the prelude.

use crate::parser::ast::{Expr, ExprKind};

/// How a `sprintf`-family call is routed to the prelude.
pub(super) enum PrintfRoute {
    /// Call the named prelude function with the same arguments.
    Rename(&'static str),
    /// Call the named prelude function with one array holding every argument (used when the
    /// call spreads an array, which cannot bind to the fixed `$format` parameter).
    Packed(&'static str),
}

/// Returns how a `sprintf`-family call must be rewritten, or `None` when the call is not one
/// of them or the native formatter handles it exactly.
pub(super) fn printf_prelude_target(function_name: &str, args: &[Expr]) -> Option<PrintfRoute> {
    let lower = function_name.to_ascii_lowercase();
    if args.iter().any(|arg| matches!(arg.kind, ExprKind::Spread(_))) {
        return match lower.as_str() {
            "sprintf" => Some(PrintfRoute::Packed("__elephc_sprintf_all")),
            "printf" => Some(PrintfRoute::Packed("__elephc_printf_all")),
            "fprintf" => Some(PrintfRoute::Packed("__elephc_fprintf_all")),
            _ => None,
        };
    }
    // `explode($separator, $string, $limit)`: the native splitter takes no limit, so the
    // limited form is the prelude's elephc-PHP wrapper around it.
    if lower == "explode"
        && args.len() == 3
        && !args.iter().any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. }))
    {
        return Some(PrintfRoute::Rename("__elephc_explode_limit"));
    }
    // `max($array)` / `min($array)`: the native helpers only compare scalar arguments, so the
    // one-argument (array) form is the prelude's elephc-PHP scan with PHP's comparison rules
    // (the first of equal values wins, an empty array is a `ValueError`).
    if (lower == "max" || lower == "min")
        && args.len() == 1
        && !matches!(args[0].kind, ExprKind::NamedArg { .. })
    {
        // A literal `[]` always throws; its `never` elements must not reach the comparison.
        let empty_literal = matches!(&args[0].kind, ExprKind::ArrayLiteral(items) if items.is_empty());
        return Some(PrintfRoute::Rename(match (lower == "max", empty_literal) {
            (true, false) => "__elephc_array_max",
            (false, false) => "__elephc_array_min",
            (true, true) => "__elephc_array_max_empty",
            (false, true) => "__elephc_array_min_empty",
        }));
    }
    let target = match lower.as_str() {
        "sprintf" => "__elephc_sprintf",
        "printf" => "__elephc_printf",
        "vsprintf" => return Some(PrintfRoute::Rename("__elephc_vsprintf")),
        "vprintf" => return Some(PrintfRoute::Rename("__elephc_vprintf")),
        "fprintf" => return Some(PrintfRoute::Rename("__elephc_fprintf")),
        "vfprintf" => return Some(PrintfRoute::Rename("__elephc_vfprintf")),
        _ => return None,
    };
    if native_format_handles(args) {
        None
    } else {
        Some(PrintfRoute::Rename(target))
    }
}

/// Returns whether a `sprintf`/`printf` argument list has a literal format the native
/// formatter reproduces byte for byte, with enough positional value arguments.
fn native_format_handles(args: &[Expr]) -> bool {
    if args
        .iter()
        .any(|arg| matches!(arg.kind, ExprKind::Spread(_) | ExprKind::NamedArg { .. }))
    {
        return false;
    }
    let Some(ExprKind::StringLiteral(format)) = args.first().map(|arg| &arg.kind) else {
        return false;
    };
    match native_conversion_count(format.as_bytes()) {
        Some(count) => count < args.len(),
        None => false,
    }
}

/// Counts the argument-consuming conversions of a format when every one of them is in the
/// natively exact subset; `None` otherwise.
fn native_conversion_count(format: &[u8]) -> Option<usize> {
    let mut count = 0;
    let mut i = 0;
    while i < format.len() {
        if format[i] != b'%' {
            i += 1;
            continue;
        }
        i += 1;
        if format.get(i) == Some(&b'%') {
            i += 1;
            continue;
        }
        let spec_start = i;
        let mut zero = false;
        while let Some(&flag) = format.get(i) {
            match flag {
                b'-' => {}
                b'0' => zero = true,
                b'+' => {}
                _ => break,
            }
            i += 1;
        }
        let width_start = i;
        while format.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        // One native conversion is bounded to 4 KiB of output.
        if i - width_start > 3 {
            return None;
        }
        let mut precision = false;
        if format.get(i) == Some(&b'.') {
            precision = true;
            i += 1;
            let start = i;
            while format.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            let digits = std::str::from_utf8(&format[start..i]).ok()?;
            if digits.is_empty() || digits.len() > 2 || digits.parse::<u32>().ok()? > 53 {
                return None;
            }
        }
        let bare = i == spec_start;
        match format.get(i)? {
            // PHP ignores a precision on integers (`%.2x` even prints nothing).
            b'd' | b'x' | b'X' | b'o' if !precision => {}
            // PHP's `%c` takes no width or flags; a bare `%s` is copied byte for byte
            // (libc stops at NUL bytes and pads strings with spaces where PHP honours `0`).
            b'c' | b's' if bare => {}
            // PHP keeps zero padding on the right of a left-aligned float, and a zero-padded
            // INF/NaN gets its first letter replaced by the sign; libc does neither.
            b'f' if !zero => {}
            _ => return None,
        }
        i += 1;
        count += 1;
    }
    Some(count)
}
