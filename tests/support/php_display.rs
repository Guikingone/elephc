//! Purpose:
//! Shared test helper that strips PHP `display_errors` diagnostic lines from captured stdout.
//!
//! Called from:
//! - The standalone integration-test binaries that capture a compiled program's output.
//!
//! Key details:
//! - The runtime mirrors php's `Warning: <message> in <file> on line <N>` line on stdout; tests
//!   that assert exact program output must not see it, while the stderr copy remains asserted.

/// Removes PHP `display_errors` diagnostic lines (`Warning: … in … on line N`) from stdout.
///
/// The diagnostic can be appended after other output on the same line, so this scans for a
/// `Warning: `/`Notice: `/`Deprecated: ` prefix and drops the whole run up to and including its
/// terminating newline. Only a run whose tail is ` on line <digits>` and whose middle carries
/// ` in ` is treated as a display diagnostic, so ordinary program text is left untouched.
#[allow(dead_code)]
pub fn strip_php_display_lines(stdout: &str) -> String {
    const PREFIXES: [&str; 3] = ["Warning: ", "Notice: ", "Deprecated: "];
    let mut out = String::with_capacity(stdout.len());
    let mut i = 0;
    'scan: while i < stdout.len() {
        if PREFIXES.iter().any(|prefix| stdout[i..].starts_with(prefix)) {
            if let Some(end) = stdout[i..].find('\n') {
                let candidate = &stdout[i..i + end];
                if let Some((head, line_number)) = candidate.rsplit_once(" on line ") {
                    if head.contains(" in ")
                        && !line_number.is_empty()
                        && line_number.bytes().all(|b| b.is_ascii_digit())
                    {
                        i += end + 1;
                        continue 'scan;
                    }
                }
            }
        }
        let ch = stdout[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}
