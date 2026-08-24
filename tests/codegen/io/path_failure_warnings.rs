//! Purpose:
//! Integration tests for the line php prints when a filesystem PATH OPERATION fails.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - MEASURED before the fix: a program of sixteen failing path calls drew SIXTEEN warnings from
//!   `php -n` 8.5.6 and NINE from elephc — and two of those nine named the wrong function.
//!   Seven builtins failed in complete silence: `unlink`, `rmdir`, `rename`, `mkdir`, `opendir`,
//!   `touch` and `chmod`. A script that checked the return value behaved the same either way; a
//!   script that read the log learned nothing at all.
//! - THE SHAPES ARE NOT ONE SHAPE, which is the point of testing them together:
//!   `unlink(path): reason`, `opendir(path): Failed to open directory: reason`,
//!   `rename(from,to): reason` — comma, no space — and `mkdir(): reason`, `chmod(): reason`,
//!   `touch(): Unable to create file PATH because reason`, whose parentheses stay EMPTY even
//!   though a path was passed.
//! - `readfile()` and `file()` read through `file_get_contents`, and left to itself that helper
//!   names ITSELF: a missing file was reported as `file_get_contents(x.txt)` under both names.
//! - The errno varies on purpose — `ENOENT`, `EEXIST`, `ENOTDIR` — because the reason text comes
//!   from the system and only the frame around it is elephc's to get right.
//! - Every expectation was measured on `php -n` 8.5.6.

use crate::support::*;

/// Verifies that each failing path builtin says WHY, in php's wording for that builtin.
#[test]
fn a_failing_path_call_says_why_the_way_php_does() {
    let out = compile_and_run_capture(
        r#"<?php
mkdir("exists");
file_put_contents("f.txt", "x");
mkdir("exists");
mkdir("/no/such/dir/deep");
rmdir("f.txt");
rmdir("exists/nope");
unlink("nope.txt");
rename("nope.txt", "other.txt");
touch("/no/such/dir/x.txt");
chmod("nope.txt", 0644);
opendir("f.txt");
opendir("nope");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.diagnostics,
        "Warning: mkdir(): File exists\n\
         Warning: mkdir(): No such file or directory\n\
         Warning: rmdir(f.txt): Not a directory\n\
         Warning: rmdir(exists/nope): No such file or directory\n\
         Warning: unlink(nope.txt): No such file or directory\n\
         Warning: rename(nope.txt,other.txt): No such file or directory\n\
         Warning: touch(): Unable to create file /no/such/dir/x.txt because No such file or directory\n\
         Warning: chmod(): No such file or directory\n\
         Warning: opendir(f.txt): Failed to open directory: Not a directory\n\
         Warning: opendir(nope): Failed to open directory: No such file or directory\n",
        "each builtin warns in ITS wording, and in the order the program calls them"
    );
}

/// Verifies that a delegating reader names ITSELF, not the helper it reads through.
///
/// `readfile()` and `file()` both go through `file_get_contents`, and both reported that name.
#[test]
fn a_delegating_reader_names_itself_in_its_warning() {
    let out = compile_and_run_capture(
        r#"<?php
readfile("/no/such/dir/x.txt");
file("/no/such/dir/x.txt");
file_get_contents("/no/such/dir/x.txt");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.diagnostics,
        "Warning: readfile(/no/such/dir/x.txt): Failed to open stream: No such file or directory\n\
         Warning: file(/no/such/dir/x.txt): Failed to open stream: No such file or directory\n\
         Warning: file_get_contents(/no/such/dir/x.txt): Failed to open stream: No such file or directory\n",
        "the function the USER called is the one php names"
    );
}

/// Verifies that `@` still silences every one of these lines.
///
/// A warning that ignores the suppression operator is worse than no warning: it appears in
/// output a program deliberately kept clean.
#[test]
fn the_suppression_operator_silences_them_all() {
    let out = compile_and_run_capture(
        r#"<?php
@mkdir("/no/such/dir/deep");
@rmdir("nope");
@unlink("nope.txt");
@rename("nope.txt", "other.txt");
@touch("/no/such/dir/x.txt");
@chmod("nope.txt", 0644);
@opendir("nope");
@readfile("nope.txt");
echo "quiet\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.diagnostics, "", "every one of these honours @");
    assert_eq!(out.stdout, "quiet\n");
}
