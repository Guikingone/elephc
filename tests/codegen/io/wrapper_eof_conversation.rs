//! Purpose:
//! Integration tests for WHEN php asks a userspace wrapper about end-of-file, and when it answers
//! from what it was already told.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A wrapper cannot set the stream's end-of-file state itself, so php asks `stream_eof()`
//!   straight after every `stream_read()` and keeps the answer. `feof()` reads that rather than
//!   asking the class again; a seek is what clears it.
//! - elephc asked nothing after its read and asked the WRAPPER at every `feof()`. The values were
//!   always right — this is about the conversation the class actually sees, which is what a
//!   wrapper with side effects or a cost per call observes.
//! - Every expectation MEASURED on `php -n` 8.5.6.

use crate::support::*;

/// A wrapper that announces every call it receives, with the position it is at.
const WRAPPER: &str = r#"<?php
class T {
    public $context;
    public $pos = 0;
    public $data = "abcdefghij";
    public function stream_open($p, $m, $o, &$x) { return true; }
    public function stream_read($n) {
        $r = substr($this->data, $this->pos, $n);
        $this->pos += strlen($r);
        echo "read\n";
        return $r;
    }
    public function stream_eof() { echo "eof\n"; return $this->pos >= strlen($this->data); }
    public function stream_seek($o, $w) { echo "seek\n"; $this->pos = $o; return true; }
    public function stream_tell() { return $this->pos; }
    public function stream_stat() { return []; }
    public function stream_close() {}
}
stream_wrapper_register("tw", "T");
"#;

/// Compiles `WRAPPER` followed by `body` and returns only the calls the class saw.
fn calls(body: &str) -> Vec<String> {
    let out = compile_and_run_capture(&format!("{WRAPPER}{body}\n"));
    assert!(out.success, "program failed: {}", out.stderr);
    out.stdout
        .lines()
        .filter(|l| matches!(*l, "read" | "eof" | "seek"))
        .map(str::to_string)
        .collect()
}

/// Verifies a read is followed by the question, and a second read served from the buffer is not.
///
/// elephc asked nothing here: the class was read and never told what php told it.
#[test]
fn test_a_read_is_followed_by_the_question() {
    assert_eq!(
        calls(r#"$h = fopen("tw://x", "r"); fread($h, 4); fread($h, 3); fclose($h);"#),
        vec!["read", "eof"],
    );
}

/// Verifies `feof()` answers from what the read was told, instead of asking again.
///
/// The whole-file readers drain the stream and then ask whether it is done; php already knows.
#[test]
fn test_feof_answers_from_what_the_read_was_told() {
    assert_eq!(
        calls(r#"$h = fopen("tw://x", "r"); fread($h, 20); var_dump(feof($h)); fclose($h);"#),
        vec!["read", "eof"],
    );
}

/// Verifies `feof()` on a stream nothing has read yet DOES ask — there is nothing to answer from.
#[test]
fn test_feof_before_any_read_asks_the_class() {
    assert_eq!(
        calls(r#"$h = fopen("tw://x", "r"); var_dump(feof($h)); fclose($h);"#),
        vec!["eof"],
    );
}

/// Verifies a seek makes the next `feof()` ask again.
///
/// The remembered answer describes a position the stream has left, so php discards it. Without
/// this the flag would outlive its own truth.
#[test]
fn test_a_seek_makes_the_next_feof_ask_again() {
    assert_eq!(
        calls(
            r#"$h = fopen("tw://x", "r"); fread($h, 20); fseek($h, 0); var_dump(feof($h)); fclose($h);"#
        ),
        vec!["read", "eof", "seek", "eof"],
    );
}

/// Verifies `file_get_contents()` sees one read and one question per fill.
#[test]
fn test_a_whole_file_read_asks_once_per_fill() {
    assert_eq!(
        calls(r#"file_get_contents("tw://y");"#),
        vec!["read", "eof", "read", "eof"],
    );
}

/// Verifies a wrapper that hands back SMALL pieces is not declared finished by the size of them.
///
/// The chunked reader judges a short read as end-of-file, which is the only thing backends that
/// cannot be asked have. A class CAN be asked, and answering from the guess instead would stop
/// `file_get_contents()` after the first three bytes.
#[test]
fn test_a_wrapper_that_hands_back_small_pieces_is_not_cut_short() {
    let out = compile_and_run_capture(
        r#"<?php
class S {
    public $context;
    public $pos = 0;
    public $data = "abcdefghij";
    public function stream_open($p, $m, $o, &$x) { return true; }
    public function stream_read($n) {
        $r = substr($this->data, $this->pos, min($n, 3));
        $this->pos += strlen($r);
        return $r;
    }
    public function stream_eof() { return $this->pos >= strlen($this->data); }
    public function stream_stat() { return []; }
    public function stream_close() {}
}
stream_wrapper_register("sw", "S");
var_dump(file_get_contents("sw://x"));
$h = fopen("sw://x", "r");
$out = "";
while (!feof($h)) { $out .= fread($h, 10); }
var_dump($out);
fclose($h);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "string(10) \"abcdefghij\"\nstring(10) \"abcdefghij\"\n"
    );
}
