//! Purpose:
//! Routes `sprintf`-family calls the native formatter cannot reproduce exactly to the
//! elephc-PHP implementation in `crate::printf_prelude`.
//!
//! Called from:
//! - `crate::name_resolver::expressions::resolve_expr` for every resolved builtin call.
//!
//! Key details:
//! - The native `__rt_sprintf` path stays for literal formats whose every conversion is a
//!   plain `%d`/`%s`/`%f`/`%x`/`%X`/`%o`/`%c` with `-`/`+`/`0` flags, a width and a decimal
//!   precision, where libc's output equals PHP's and each argument is converted at compile
//!   time; everything else (dynamic formats, `%1$s`, `*`, `'c` padding, `b`/`u`/`e`/`g`/`F`,
//!   the `l` modifier, spread or named arguments, too few arguments) goes to the prelude.
//! - `vsprintf`, `vprintf`, `fprintf` and `vfprintf` always use the prelude.

use crate::parser::ast::{Expr, ExprKind};

/// Returns the prelude function a `sprintf`-family call must be rewritten to, or `None` when
/// the call is not one of them or the native formatter handles it exactly.
pub(super) fn printf_prelude_target(function_name: &str, args: &[Expr]) -> Option<&'static str> {
    let target = match function_name.to_ascii_lowercase().as_str() {
        "sprintf" => "__elephc_sprintf",
        "printf" => "__elephc_printf",
        "vsprintf" => return Some("__elephc_vsprintf"),
        "vprintf" => return Some("__elephc_vprintf"),
        "fprintf" => return Some("__elephc_fprintf"),
        "vfprintf" => return Some("__elephc_vfprintf"),
        _ => return None,
    };
    if native_format_handles(args) {
        None
    } else {
        Some(target)
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
        if format.get(i) == Some(&b'.') {
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
        match format.get(i)? {
            b'd' | b'x' | b'X' | b'o' | b'c' => {}
            // libc pads strings with spaces where PHP honours `0`.
            b's' if !zero => {}
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
