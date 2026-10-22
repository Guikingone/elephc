//! Purpose:
//! Shares PHP Reflection's scalar dump formatting between AOT metadata and eval.
//!
//! Called from:
//! - The compiler's callable dump renderer and Magician's Reflection formatter.
//!
//! Key details:
//! - Reflection uses 14 significant decimal digits and bytewise printable string escapes.
//! - These helpers depend on neither backend nor runtime cells.

/// Formats a float using PHP Reflection's default precision, exponent range and spelling.
pub fn float(value: f64) -> String {
    if value.is_nan() { return "NAN".into(); }
    if value.is_infinite() {
        return if value.is_sign_negative() { "-INF" } else { "INF" }.into();
    }
    // Round before choosing the notation: rounding can carry into the next exponent.
    let rounded = format!("{:.13e}", value.abs());
    let (mantissa, exponent) = rounded.split_once('e').expect("scientific float");
    let exponent: i32 = exponent.parse().expect("decimal exponent");
    let digits = mantissa.replace('.', "");
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if !(-4..14).contains(&exponent) {
        let fraction = if digits.len() > 1 { &digits[1..] } else { "0" };
        return format!("{sign}{}.{fraction}E{exponent:+}", &digits[..1]);
    }
    let decimal = exponent + 1;
    if decimal <= 0 {
        return format!("{sign}0.{}{digits}", "0".repeat((-decimal) as usize));
    }
    let decimal = decimal as usize;
    if decimal >= digits.len() {
        format!("{sign}{digits}{}.0", "0".repeat(decimal - digits.len()))
    } else {
        format!("{sign}{}.{}", &digits[..decimal], &digits[decimal..])
    }
}

/// Quotes a string using PHP Reflection's bytewise escapes, preserving literal quote marks.
pub fn string(value: &[u8]) -> String {
    let mut result = String::from("'");
    for &byte in value {
        match byte {
            b'\t' => result.push_str("\\t"), b'\n' => result.push_str("\\n"),
            b'\r' => result.push_str("\\r"), 0x0b => result.push_str("\\v"),
            0x0c => result.push_str("\\f"), 0x1b => result.push_str("\\e"),
            b'\\' => result.push_str("\\\\"), 0x20..=0x7e => result.push(byte as char),
            _ => result.push_str(&format!("\\x{byte:02X}")),
        }
    }
    result.push('\'');
    result
}

/// Renders one named type, including PHP's expansion of the iterable alias.
pub fn named_type(name: &str, allows_null: bool) -> String {
    if name == "iterable" {
        return if allows_null { "Traversable|array|null" } else { "Traversable|array" }.into();
    }
    if allows_null && !matches!(name, "mixed" | "null") {
        format!("?{name}")
    } else {
        name.to_string()
    }
}

/// Sorts a union in PHP's type-mask order while preserving the order of class names.
pub fn union(mut names: Vec<String>, allows_null: bool) -> String {
    if let Some(index) = names.iter().position(|name| name == "iterable") {
        names.splice(index..=index, ["Traversable".into(), "array".into()]);
    }
    if allows_null && !names.iter().any(|name| name == "null") { names.push("null".into()); }
    names.sort_by_key(|name| match name.as_str() {
        "static" => 1, "callable" => 2, "object" => 3, "array" => 4,
        "string" => 6, "int" => 7, "float" => 8, "bool" => 9,
        "false" => 10, "true" => 11, "null" => 12, _ => 0,
    });
    names.dedup();
    names.join("|")
}

#[cfg(test)]
mod tests {
    /// Rounding controls the notation boundary and preserves negative zero and non-finite values.
    #[test]
    fn reflection_float_precision_and_boundaries() {
        for (value, expected) in [
            (0.30000000000000004, "0.3"), (1e14, "1.0E+14"),
            (1e13, "10000000000000.0"), (1e-4, "0.0001"), (1e-5, "1.0E-5"),
            (99999999999999.9, "1.0E+14"), (0.00009999999999999999, "0.0001"),
            (-0.0, "-0.0"), (f64::INFINITY, "INF"), (f64::NEG_INFINITY, "-INF"),
            (f64::NAN, "NAN"),
        ] {
            assert_eq!(super::float(value), expected, "{value:?}");
        }
    }

    /// Control bytes, UTF-8 and quotes follow PHP's printable dump representation.
    #[test]
    fn reflection_string_escapes() {
        assert_eq!(super::string(b"\xff\0\n\t\\'"), "'\\xFF\\x00\\n\\t\\\\''");
        assert_eq!(super::string("é".as_bytes()), "'\\xC3\\xA9'");
    }
}
