//! Purpose:
//! Integration or regression tests for diagnostic coverage of I/O builtin filesystem, including file get contents wrong args, file get contents false return rejects string return type, and file put contents wrong args.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

/// Verifies `file_get_contents()` rejects zero arguments with arity error.
#[test]
fn test_error_file_get_contents_wrong_args() {
    expect_error(
        "<?php file_get_contents();",
        "file_get_contents() takes exactly 1 argument",
    );
}

/// Verifies returning `file_get_contents()` (typed `Str|False`) from a `: string` function is
/// ACCEPTED under PHP weak-mode `string`-boundary coercion: the boxed-`Mixed` return boundary
/// runs `__rt_mixed_cast_string`, which maps the `false` failure marker to `""` (matching
/// non-`strict_types` PHP), so it flows into the scalar `string` return instead of erroring. This
/// is the shape Symfony's non-strict Dotenv/Path/Yaml boundaries rely on. (The `int|false`→`:int`
/// sentinel diagnostic stays loud — see `test_error_readfile_false_return_into_int_return_type` —
/// because a silent `false`→`0` there hides the classic `0`-is-a-valid-offset footgun.)
#[test]
fn test_file_get_contents_false_return_coerces_into_string_return_type() {
    expect_ok(
        r#"<?php
function read_file(): string {
    return file_get_contents("missing.txt");
}
"#,
    );
}

/// Verifies `hash_file()` rejects too few arguments with arity error.
#[test]
fn test_error_hash_file_wrong_args() {
    expect_error(
        r#"<?php hash_file("sha256");"#,
        "hash_file() takes 2 or 3 arguments",
    );
}

/// Verifies `readfile()` rejects zero arguments with arity error.
#[test]
fn test_error_readfile_wrong_args() {
    expect_error("<?php readfile();", "readfile() takes exactly 1 argument");
}

/// Verifies the gradual-typing boundary model accepts returning `readfile()` (typed `Int|Bool`)
/// from an `: int` function: `Bool` is PHP-coercible to `int` (weak mode coerces `false` to `0`),
/// so the union flows into the scalar return with a runtime boundary guard instead of erroring.
#[test]
fn test_error_readfile_false_return_into_int_return_type() {
    expect_error(
        r#"<?php
function dump_file(): int {
    return readfile("missing.txt");
}
"#,
        "Function 'dump_file' return type expects Int, got Union([Int, False])",
    );
}

/// Verifies `file_put_contents()` rejects one argument (PHP allows 2–4) with arity error.
#[test]
fn test_error_file_put_contents_wrong_args() {
    expect_error(
        r#"<?php file_put_contents("x");"#,
        "file_put_contents() takes 2 to 4 arguments",
    );
}

/// Verifies `file_put_contents()` rejects five arguments (PHP allows 2–4) with arity error.
#[test]
fn test_error_file_put_contents_too_many_args() {
    expect_error(
        r#"<?php file_put_contents("x", "y", 0, null, 1);"#,
        "file_put_contents() takes 2 to 4 arguments",
    );
}

/// Verifies `file_exists()` rejects zero arguments with arity error.
#[test]
fn test_error_file_exists_wrong_args() {
    expect_error(
        "<?php file_exists();",
        "file_exists() takes exactly 1 argument",
    );
}

/// Verifies `mkdir()` rejects zero arguments with arity error.
#[test]
fn test_error_mkdir_wrong_args() {
    // mkdir() gained optional $permissions/$recursive/$context params since
    // 3a2bb667a; zero args is still invalid (directory is required), just
    // with a "1 to 4" range message now.
    expect_error("<?php mkdir();", "mkdir() takes 1 to 4 arguments");
}

/// Verifies `copy()` rejects one argument (requires 2) with arity error.
#[test]
fn test_error_copy_wrong_args() {
    expect_error(r#"<?php copy("x");"#, "copy() takes exactly 2 arguments");
}

/// Verifies `link()` rejects one argument (requires 2) with arity error.
#[test]
fn test_error_link_wrong_args() {
    expect_error(r#"<?php link("x");"#, "link() takes exactly 2 arguments");
}

/// Verifies `symlink()` rejects one argument (requires 2) with arity error.
#[test]
fn test_error_symlink_wrong_args() {
    expect_error(
        r#"<?php symlink("target");"#,
        "symlink() takes exactly 2 arguments",
    );
}

/// Verifies `readlink()` rejects zero arguments with arity error.
#[test]
fn test_error_readlink_wrong_args() {
    expect_error("<?php readlink();", "readlink() takes exactly 1 argument");
}

/// Verifies `linkinfo()` rejects zero arguments with arity error.
#[test]
fn test_error_linkinfo_wrong_args() {
    expect_error("<?php linkinfo();", "linkinfo() takes exactly 1 argument");
}

/// Verifies `rename()` rejects one argument (requires 2) with arity error.
#[test]
fn test_error_rename_wrong_args() {
    expect_error(
        r#"<?php rename("x");"#,
        "rename() takes exactly 2 arguments",
    );
}

/// Verifies `getcwd()` rejects arguments with arity error.
#[test]
fn test_error_getcwd_wrong_args() {
    expect_error("<?php getcwd(1);", "getcwd() takes no arguments");
}

/// Verifies `scandir()` rejects zero arguments with arity error.
#[test]
fn test_error_scandir_wrong_args() {
    // scandir() gained optional $sorting_order/$context params since
    // 3a2bb667a; zero args is still invalid (directory is required), just
    // with a "1 to 3" range message now.
    expect_error("<?php scandir();", "scandir() takes 1 to 3 arguments");
}

/// Verifies `tempnam()` rejects one argument (requires 2) with arity error.
#[test]
fn test_error_tempnam_wrong_args() {
    expect_error(
        r#"<?php tempnam("x");"#,
        "tempnam() takes exactly 2 arguments",
    );
}

/// Verifies `is_file()` rejects zero arguments with arity error.
#[test]
fn test_error_is_file_wrong_args() {
    expect_error("<?php is_file();", "is_file() takes exactly 1 argument");
}

/// Verifies `is_dir()` rejects zero arguments with arity error.
#[test]
fn test_error_is_dir_wrong_args() {
    expect_error("<?php is_dir();", "is_dir() takes exactly 1 argument");
}

/// Verifies `is_readable()` rejects zero arguments with arity error.
#[test]
fn test_error_is_readable_wrong_args() {
    expect_error(
        "<?php is_readable();",
        "is_readable() takes exactly 1 argument",
    );
}

/// Verifies `is_writable()` rejects zero arguments with arity error.
#[test]
fn test_error_is_writable_wrong_args() {
    expect_error(
        "<?php is_writable();",
        "is_writable() takes exactly 1 argument",
    );
}

/// Verifies `filesize()` rejects zero arguments with arity error.
#[test]
fn test_error_filesize_wrong_args() {
    expect_error("<?php filesize();", "filesize() takes exactly 1 argument");
}

/// Verifies `filemtime()` rejects zero arguments with arity error.
#[test]
fn test_error_filemtime_wrong_args() {
    expect_error("<?php filemtime();", "filemtime() takes exactly 1 argument");
}

/// Verifies arity errors for extended stat builtins: fileatime, filectime, fileperms,
/// fileowner, filegroup, fileinode, filetype, is_executable, is_link, is_writeable,
/// stat, lstat, fstat, and clearstatcache with too many args.
#[test]
fn test_error_extended_stat_builtins_wrong_args() {
    for (source, message) in [
        ("<?php fileatime();", "fileatime() takes exactly 1 argument"),
        ("<?php filectime();", "filectime() takes exactly 1 argument"),
        ("<?php fileperms();", "fileperms() takes exactly 1 argument"),
        ("<?php fileowner();", "fileowner() takes exactly 1 argument"),
        ("<?php filegroup();", "filegroup() takes exactly 1 argument"),
        ("<?php fileinode();", "fileinode() takes exactly 1 argument"),
        ("<?php filetype();", "filetype() takes exactly 1 argument"),
        ("<?php is_executable();", "is_executable() takes exactly 1 argument"),
        ("<?php is_link();", "is_link() takes exactly 1 argument"),
        ("<?php is_writeable();", "is_writeable() takes exactly 1 argument"),
        ("<?php stat();", "stat() takes exactly 1 argument"),
        ("<?php lstat();", "lstat() takes exactly 1 argument"),
        ("<?php fstat();", "fstat() takes exactly 1 argument"),
        (
            "<?php clearstatcache(false, \"a\", \"extra\");",
            "clearstatcache() takes at most 2 arguments",
        ),
    ] {
        expect_error(source, message);
    }
}

/// Verifies `unlink()` rejects zero arguments with arity error.
#[test]
fn test_error_unlink_wrong_args() {
    expect_error("<?php unlink();", "unlink() takes exactly 1 argument");
}

/// Verifies `rmdir()` rejects zero arguments with arity error.
#[test]
fn test_error_rmdir_wrong_args() {
    expect_error("<?php rmdir();", "rmdir() takes exactly 1 argument");
}

/// Verifies `chdir()` rejects zero arguments with arity error.
#[test]
fn test_error_chdir_wrong_args() {
    expect_error("<?php chdir();", "chdir() takes exactly 1 argument");
}

/// Verifies `glob()` rejects zero arguments with arity error.
#[test]
fn test_error_glob_wrong_args() {
    // glob() gained an optional $flags param since 3a2bb667a; zero args is
    // still invalid (pattern is required), just with a "1 or 2" message now.
    expect_error("<?php glob();", "glob() takes 1 or 2 arguments");
}

/// Verifies `sys_get_temp_dir()` rejects arguments with arity error.
#[test]
fn test_error_sys_get_temp_dir_wrong_args() {
    expect_error(
        "<?php sys_get_temp_dir(1);",
        "sys_get_temp_dir() takes no arguments",
    );
}

/// Verifies the invalid-call diagnostic for error disk free space wrong args.
#[test]
fn test_error_disk_free_space_wrong_args() {
    expect_error(
        "<?php disk_free_space();",
        "disk_free_space() takes exactly 1 argument",
    );
}

/// Verifies the invalid-call diagnostic for error disk total space wrong args.
#[test]
fn test_error_disk_total_space_wrong_args() {
    expect_error(
        "<?php disk_total_space();",
        "disk_total_space() takes exactly 1 argument",
    );
}
