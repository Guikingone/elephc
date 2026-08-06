//! Purpose:
//! Records compile-phase durations and optional notes for CLI timing output.
//! Reports each successful phase to the interactive progress renderer while
//! retaining the full timing table only when the user requests `--timings`.
//!
//! Called from:
//! - `crate::pipeline::compile()` around each major compiler phase.
//!
//! Key details:
//! - Phase durations are always measured for interactive completed-step lines.
//! - Disabled timing skips only table storage/reporting, so pipeline code does
//!   not branch around every measurement.

use std::time::{Duration, Instant};

/// Compile timing collector for optional performance profiling.
pub(crate) struct CompileTimings {
    enabled: bool,
    started_at: Instant,
    notes: Vec<String>,
    phases: Vec<(&'static str, Duration)>,
}

/// Box-drawing characters used by the interactive and plain timing tables.
struct TableChars {
    horizontal: char,
    vertical: char,
    top_left: char,
    top_join: char,
    top_right: char,
    middle_left: char,
    middle_join: char,
    middle_right: char,
    bottom_left: char,
    bottom_join: char,
    bottom_right: char,
}

const UNICODE_TABLE: TableChars = TableChars {
    horizontal: '─',
    vertical: '│',
    top_left: '┌',
    top_join: '┬',
    top_right: '┐',
    middle_left: '├',
    middle_join: '┼',
    middle_right: '┤',
    bottom_left: '└',
    bottom_join: '┴',
    bottom_right: '┘',
};

const ASCII_TABLE: TableChars = TableChars {
    horizontal: '-',
    vertical: '|',
    top_left: '+',
    top_join: '+',
    top_right: '+',
    middle_left: '+',
    middle_join: '+',
    middle_right: '+',
    bottom_left: '+',
    bottom_join: '+',
    bottom_right: '+',
};

impl CompileTimings {
    /// Creates a new timing collector.
    ///
    /// `enabled` controls whether detailed timing data is retained and reported.
    /// Phase completion still reaches the interactive progress renderer when
    /// disabled. The internal timer starts immediately.
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            started_at: Instant::now(),
            notes: Vec::new(),
            phases: Vec::new(),
        }
    }

    /// Records the elapsed time since `started_at` for the named phase.
    ///
    /// The duration is always forwarded to the interactive progress renderer.
    /// It is retained for the detailed timing report only when collection is
    /// enabled.
    pub(crate) fn record_since(&mut self, phase: &'static str, started_at: Instant) {
        let elapsed = started_at.elapsed();
        crate::progress::finish_phase(phase, elapsed);
        if self.enabled {
            self.phases.push((phase, elapsed));
        }
    }

    /// Appends an arbitrary informational note to the timing report.
    ///
    /// No-op when timing collection is disabled. The note is printed verbatim
    /// in order below the timing table.
    pub(crate) fn note(&mut self, note: impl Into<String>) {
        if self.enabled {
            self.notes.push(note.into());
        }
    }

    /// Returns elapsed time since this collector was constructed, regardless of
    /// whether timing collection is enabled. Used for the final success line's
    /// elapsed-seconds suffix even when `--timings` was not passed.
    pub(crate) fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Prints the collected timing report to stderr.
    ///
    /// Output is gated behind the `enabled` flag. The report renders friendly
    /// phase labels, adaptive durations, percentage shares, and a total row.
    /// Interactive runs use Unicode box drawing and bold header/total rows;
    /// plain runs use an unstyled ASCII table suitable for logs and pipes.
    pub(crate) fn report(&self) {
        if !self.enabled {
            return;
        }

        let decorated = crate::progress::is_decorated();
        let total = self.started_at.elapsed();

        eprintln!("{}", style_if(decorated, "Compiler timings"));
        eprintln!("{}", render_timing_table(&self.phases, total, decorated));
        if !self.notes.is_empty() {
            eprintln!();
            eprintln!("{}", style_if(decorated, "Notes"));
            let bullet = if decorated { "•" } else { "-" };
            for note in &self.notes {
                eprintln!("  {bullet} {note}");
            }
        }
    }
}

/// Returns `part`'s share of `total` as a percentage, or `0.0` when `total` is
/// zero (guards the first phase recorded before any measurable time elapses).
fn percentage(part: Duration, total: Duration) -> f64 {
    if total.as_secs_f64() == 0.0 {
        0.0
    } else {
        part.as_secs_f64() / total.as_secs_f64() * 100.0
    }
}

/// Applies bold terminal styling only for decorated interactive runs.
fn style_if(decorated: bool, text: &str) -> String {
    if decorated {
        console::style(text).bold().to_string()
    } else {
        text.to_string()
    }
}

/// Renders the complete timing table using terminal-appropriate borders.
fn render_timing_table(
    phases: &[(&str, Duration)],
    total: Duration,
    decorated: bool,
) -> String {
    let chars = if decorated {
        &UNICODE_TABLE
    } else {
        &ASCII_TABLE
    };
    let rows: Vec<(String, String, String)> = phases
        .iter()
        .map(|(phase, duration)| {
            (
                crate::progress::phase_label(phase).to_string(),
                crate::progress::format_phase_duration(*duration),
                format!("{:.1}%", percentage(*duration, total)),
            )
        })
        .collect();
    let total_duration = crate::progress::format_phase_duration(total);
    let total_share = "100.0%";

    let phase_width = rows
        .iter()
        .map(|row| row.0.chars().count())
        .fold("Phase".len().max("Total".len()), usize::max);
    let duration_width = rows
        .iter()
        .map(|row| row.1.len())
        .fold("Duration".len().max(total_duration.len()), usize::max);
    let share_width = rows
        .iter()
        .map(|row| row.2.len())
        .fold("Share".len().max(total_share.len()), usize::max);

    let mut lines = Vec::with_capacity(rows.len() + 6);
    lines.push(format_table_rule(
        chars,
        phase_width,
        duration_width,
        share_width,
        chars.top_left,
        chars.top_join,
        chars.top_right,
    ));
    let header = format_table_row(
        chars,
        "Phase",
        "Duration",
        "Share",
        phase_width,
        duration_width,
        share_width,
    );
    lines.push(style_if(decorated, &header));
    lines.push(format_table_rule(
        chars,
        phase_width,
        duration_width,
        share_width,
        chars.middle_left,
        chars.middle_join,
        chars.middle_right,
    ));
    let has_phase_rows = !rows.is_empty();
    for (phase, duration, share) in rows {
        lines.push(format_table_row(
            chars,
            &phase,
            &duration,
            &share,
            phase_width,
            duration_width,
            share_width,
        ));
    }
    if has_phase_rows {
        lines.push(format_table_rule(
            chars,
            phase_width,
            duration_width,
            share_width,
            chars.middle_left,
            chars.middle_join,
            chars.middle_right,
        ));
    }
    let total_row = format_table_row(
        chars,
        "Total",
        &total_duration,
        total_share,
        phase_width,
        duration_width,
        share_width,
    );
    lines.push(style_if(decorated, &total_row));
    lines.push(format_table_rule(
        chars,
        phase_width,
        duration_width,
        share_width,
        chars.bottom_left,
        chars.bottom_join,
        chars.bottom_right,
    ));
    lines.join("\n")
}

/// Formats one table row with left-aligned phase text and numeric columns right-aligned.
fn format_table_row(
    chars: &TableChars,
    phase: &str,
    duration: &str,
    share: &str,
    phase_width: usize,
    duration_width: usize,
    share_width: usize,
) -> String {
    let phase = format!("{phase:<phase_width$}");
    let duration = format!("{duration:>duration_width$}");
    let share = format!("{share:>share_width$}");
    format!(
        "{} {phase} {} {duration} {} {share} {}",
        chars.vertical, chars.vertical, chars.vertical, chars.vertical,
    )
}

/// Formats a horizontal table border or separator for the computed column widths.
fn format_table_rule(
    chars: &TableChars,
    phase_width: usize,
    duration_width: usize,
    share_width: usize,
    left: char,
    join: char,
    right: char,
) -> String {
    let phase = chars.horizontal.to_string().repeat(phase_width + 2);
    let duration = chars.horizontal.to_string().repeat(duration_width + 2);
    let share = chars.horizontal.to_string().repeat(share_width + 2);
    format!("{left}{phase}{join}{duration}{join}{share}{right}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies disabled detailed timings do not retain phase samples.
    #[test]
    fn disabled_timings_do_not_store_phases() {
        let mut timings = CompileTimings::new(false);
        timings.record_since("read", Instant::now());
        assert!(timings.phases.is_empty());
    }

    /// Verifies percentage formatting avoids division by zero.
    #[test]
    fn percentage_of_zero_total_is_zero() {
        assert_eq!(percentage(Duration::from_millis(5), Duration::ZERO), 0.0);
    }

    /// Verifies phase shares are computed against total elapsed time.
    #[test]
    fn percentage_computes_share_of_total() {
        let pct = percentage(Duration::from_millis(25), Duration::from_millis(100));
        assert!((pct - 25.0).abs() < 0.001);
    }

    /// Verifies plain timing titles remain unstyled.
    #[test]
    fn style_if_plain_is_unchanged() {
        assert_eq!(style_if(false, "Compiler timings"), "Compiler timings");
    }

    /// Verifies decorated timing headers retain their original text.
    #[test]
    fn style_if_decorated_contains_original_text() {
        assert!(style_if(true, "Compiler timings").contains("Compiler timings"));
    }

    /// Verifies plain reports use ASCII borders, friendly labels, and percentages.
    #[test]
    fn plain_timing_table_is_ascii_and_complete() {
        let table = render_timing_table(
            &[("read", Duration::from_millis(25))],
            Duration::from_millis(100),
            false,
        );
        assert!(table.starts_with('+'));
        assert!(table.ends_with('+'));
        assert!(table.contains("| Reading source "));
        assert!(table.contains("25.00 ms"));
        assert!(table.contains("25.0%"));
        assert!(table.contains("| Total"));
        assert!(!table.contains('┌'));
    }

    /// Verifies interactive reports use Unicode borders and adaptive durations.
    #[test]
    fn decorated_timing_table_uses_unicode_and_friendly_labels() {
        let table = render_timing_table(
            &[("ir-opt", Duration::from_millis(1_250))],
            Duration::from_millis(2_500),
            true,
        );
        assert!(table.starts_with('┌'));
        assert!(table.ends_with('┘'));
        assert!(table.contains('┼'));
        assert!(table.contains("Optimizing EIR"));
        assert!(table.contains("1.25 s"));
        assert!(table.contains("50.0%"));
    }

    /// Verifies every line in the plain table has the same visible width.
    #[test]
    fn plain_timing_table_columns_align() {
        let table = render_timing_table(
            &[
                ("read", Duration::from_micros(500)),
                ("autoload-run", Duration::from_millis(1_250)),
            ],
            Duration::from_millis(2_000),
            false,
        );
        let widths: Vec<usize> = table.lines().map(|line| line.chars().count()).collect();
        assert!(widths.windows(2).all(|pair| pair[0] == pair[1]));
    }

    /// Verifies an empty report uses one separator between the header and total row.
    #[test]
    fn empty_timing_table_has_no_adjacent_separators() {
        let table = render_timing_table(&[], Duration::ZERO, false);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 5);
        assert!(lines[1].contains("Phase"));
        assert!(lines[3].contains("Total"));
    }
}
