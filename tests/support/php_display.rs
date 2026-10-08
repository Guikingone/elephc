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
/// A diagnostic ends at the first ` on line <digits>` terminator, which may follow embedded
/// newlines carried by the message (e.g. an array key containing a newline). Its start is either a
/// `Warning: `/`Notice: `/`Deprecated: ` prefix, or the beginning of its line when the location
/// infix names a path (the eval bridge submits messages without the level prefix). The diagnostic
/// can be appended after other output on the same line, so only the matched run is dropped. The raw
/// stdout remains available as `ProgramOutput::stdout_raw`.
#[allow(dead_code)]
pub fn strip_php_display_lines(stdout: &str) -> String {
    const PREFIXES: [&str; 3] = ["Warning: ", "Notice: ", "Deprecated: "];
    const TERMINATOR: &str = " on line ";
    const INFIX: &str = " in ";
    let mut out = String::with_capacity(stdout.len());
    let mut i = 0;
    while i < stdout.len() {
        let mut terminator = None;
        let mut search = i;
        while let Some(rel) = stdout[search..].find(TERMINATOR) {
            let digits_at = search + rel + TERMINATOR.len();
            let digits: String = stdout[digits_at..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if !digits.is_empty() {
                let tail = digits_at + digits.len();
                if stdout[tail..].starts_with('\n') {
                    terminator = Some((search + rel, tail + 1));
                    break;
                }
                if tail == stdout.len() {
                    terminator = Some((search + rel, tail));
                    break;
                }
            }
            search = digits_at;
        }
        let Some((term_start, term_end)) = terminator else {
            out.push_str(&stdout[i..]);
            break;
        };
        let line_start = stdout[..term_start]
            .rfind('\n')
            .map(|position| position + 1)
            .unwrap_or(0)
            .max(i);
        let prefix_start = PREFIXES
            .iter()
            .filter_map(|prefix| {
                stdout[line_start..term_start]
                    .find(prefix)
                    .map(|off| line_start + off)
            })
            .min();
        let infix_at = stdout[line_start..term_start]
            .rfind(INFIX)
            .map(|off| line_start + off + INFIX.len());
        let path_ok = infix_at.is_some_and(|file_at| {
            let file = &stdout[file_at..term_start];
            file.contains('/') || file.ends_with(".php")
        });
        let run_start = if infix_at.is_some() && (prefix_start.is_some() || path_ok) {
            Some(prefix_start.unwrap_or(line_start))
        } else {
            None
        };
        match run_start {
            Some(run_start) if run_start >= i => {
                out.push_str(&stdout[i..run_start]);
                i = term_end;
            }
            _ => {
                out.push_str(&stdout[i..term_end]);
                i = term_end;
            }
        }
    }
    out
}
