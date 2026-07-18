//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of I/O filesystem, including mkdir rmdir, copy unlink, and rename file.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies mkdir, rmdir, and is_dir by creating a directory, confirming it
/// exists, removing it, and confirming it no longer exists.
#[test]
fn test_fread_inside_user_function_does_not_overwrite_other_locals() {
    // Regression for a frame-layout bug: when fread() was used inside a user
    // function and its result was assigned to a local variable, the codegen
    // inference fell back to PhpType::Mixed (8-byte slot) instead of Str
    // (16-byte). The store path still wrote the string as a 16-byte (ptr+len)
    // pair, so the second 8 bytes clobbered the adjacent local — typically
    // the just-opened $f resource — and the next fclose($f) crashed because
    // it tried to mixed-unbox an integer length.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("readfn.txt", "elephc");
function read_back() {
    $f = fopen("readfn.txt", "r");
    $r = fread($f, 64);
    fclose($f);
    return $r;
}
echo read_back();
unlink("readfn.txt");
"#,
    );
    assert_eq!(out, "elephc");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for mkdir rmdir.
#[test]
fn test_mkdir_rmdir() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
mkdir("testdir");
if (is_dir("testdir")) { echo "made"; }
rmdir("testdir");
if (!is_dir("testdir")) { echo "gone"; }
"#,
    );
    assert_eq!(out, "madegone");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies copy, unlink, and file existence by creating a file, copying it,
/// reading through the copy, deleting both files, and confirming removal.
#[test]
fn test_copy_unlink() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("orig.txt", "content");
copy("orig.txt", "dup.txt");
echo file_get_contents("dup.txt");
unlink("dup.txt");
if (!file_exists("dup.txt")) { echo "|gone"; }
unlink("orig.txt");
"#,
    );
    assert_eq!(out, "content|gone");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies rename by creating a file, renaming it, confirming the new name
/// holds the data, confirming the old name is gone, and cleaning up.
#[test]
fn test_rename_file() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("old.txt", "data");
rename("old.txt", "new.txt");
echo file_get_contents("new.txt");
if (!file_exists("old.txt")) { echo "|moved"; }
unlink("new.txt");
"#,
    );
    assert_eq!(out, "data|moved");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies getcwd returns a non-empty string (platform-independent check).
#[test]
fn test_getcwd() {
    let out = compile_and_run(
        r#"<?php
$cwd = getcwd();
if (strlen($cwd) > 0) { echo "ok"; }
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies sys_get_temp_dir returns a path containing "tmp" (case-insensitive
/// check to cover Linux, macOS, and Windows temp naming).
#[test]
fn test_sys_get_temp_dir() {
    let out = compile_and_run(
        r#"<?php
$tmp = sys_get_temp_dir();
echo $tmp;
"#,
    );
    assert!(out.contains("tmp") || out.contains("Tmp"));
}

/// Verifies chdir changes the working directory and getcwd reflects the new
/// path, confirming the change by checking path length increased after chdir.
#[test]
fn test_chdir_getcwd() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
mkdir("subdir");
$before = getcwd();
chdir("subdir");
$after = getcwd();
if (strlen($after) > strlen($before)) { echo "changed"; }
chdir("..");
rmdir("subdir");
"#,
    );
    assert_eq!(out, "changed");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies scandir by creating two files, confirming all four entries (. .. a.txt b.txt)
/// appear in the result, and cleaning up the directory.
#[test]
fn test_scandir() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
mkdir("sd");
file_put_contents("sd/a.txt", "a");
file_put_contents("sd/b.txt", "b");
$files = scandir("sd");
if (
    count($files) == 4 &&
    in_array(".", $files) &&
    in_array("..", $files) &&
    in_array("a.txt", $files) &&
    in_array("b.txt", $files)
) {
    echo "ok";
}
unlink("sd/a.txt");
unlink("sd/b.txt");
rmdir("sd");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: php-verified `scandir()` sort-order matrix. Default (ascending, byte
/// order not locale) fixes a pre-existing divergence (the runtime previously
/// returned raw unsorted `readdir()` order); `SCANDIR_SORT_DESCENDING` reverses
/// it; `SCANDIR_SORT_NONE` keeps raw order. `"."`/`".."` are always included
/// and participate in the sort exactly like PHP (php-verified with mixed-case
/// names: `Banana`/`Cherry`/`apple` sort byte-ascending as `Banana,Cherry,apple`).
#[test]
fn test_scandir_sort_order_matrix() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("sdsort");
file_put_contents("sdsort/Banana", "1");
file_put_contents("sdsort/apple", "1");
file_put_contents("sdsort/Cherry", "1");
$asc = scandir("sdsort");
$desc = scandir("sdsort", SCANDIR_SORT_DESCENDING);
$none = scandir("sdsort", SCANDIR_SORT_NONE);
echo implode(",", $asc), "|";
echo implode(",", $desc), "|";
$none_sorted = $none;
sort($none_sorted);
// Avoid array === (unsupported for Array(Str) on this branch) by comparing
// the imploded string forms instead.
echo (implode(",", $none_sorted) === implode(",", $asc)) ? "ok" : "mismatch";
unlink("sdsort/Banana");
unlink("sdsort/apple");
unlink("sdsort/Cherry");
rmdir("sdsort");
"#,
    );
    assert_eq!(
        out,
        ".,..,Banana,Cherry,apple|apple,Cherry,Banana,..,.|ok"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// H5: `mkdir($dir, 0777, true)` creates nested parent directories that do not
/// yet exist (php-verified real recursive semantics, not accept-and-ignore).
#[test]
fn test_mkdir_recursive_creates_nested_dirs() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
$r = mkdir("a/b/c", 0777, true);
if ($r && is_dir("a/b/c")) { echo "ok"; }
rmdir("a/b/c");
rmdir("a/b");
rmdir("a");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5 JURY ADDENDUM item 1: `mkdir()` on an existing directory returns `false`
/// (php-verified: PHP emits a warning and returns `false`, matching an
/// EEXIST-mapped mkdir()/mkdirat() failure) — both non-recursive and recursive.
#[test]
fn test_mkdir_existing_dir_returns_false() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("existing");
$r1 = mkdir("existing");
$r2 = mkdir("existing", 0777, true);
echo ($r1 === false) ? "false1" : "true1";
echo "|";
echo ($r2 === false) ? "false2" : "true2";
rmdir("existing");
"#,
    );
    assert_eq!(out, "false1|false2");
    let _ = fs::remove_dir_all(&dir);
}

/// H5 JURY ADDENDUM item 1: `mkdir()` without `$recursive` on a path with
/// missing parents returns `false` (php-verified).
#[test]
fn test_mkdir_missing_parent_non_recursive_returns_false() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
$r = mkdir("missing_parent/child");
echo ($r === false) ? "ok" : "unexpected";
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: `mkdir($dir, $mode)` passes the real requested permission bits to the
/// syscall (masked by the process umask, matching PHP), instead of the
/// previously-hardcoded 0755.
#[test]
fn test_mkdir_permissions_applied() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
umask(0);
mkdir("permdir", 0700);
$perms = fileperms("permdir") & 0777;
echo $perms === 0700 ? "ok" : (string) $perms;
rmdir("permdir");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies glob by creating two files matching a pattern, confirming both
/// are returned with their full paths, and cleaning up.
#[test]
fn test_glob_fn() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("gd");
file_put_contents("gd/g1.txt", "a");
file_put_contents("gd/g2.txt", "b");
$matches = glob("gd/*.txt");
if (
    count($matches) == 2 &&
    in_array("gd/g1.txt", $matches) &&
    in_array("gd/g2.txt", $matches)
) {
    echo "ok";
}
unlink("gd/g1.txt");
unlink("gd/g2.txt");
rmdir("gd");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: php-verified `glob($pattern, GLOB_ONLYDIR)` returns only directory
/// matches (post-filtered via `__rt_is_dir()`, never forwarded to libc as a
/// bit — see the runtime helper's module doc).
#[test]
fn test_glob_onlydir_filters_to_directories() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("gd2");
mkdir("gd2/subdir");
file_put_contents("gd2/file.txt", "x");
$matches = glob("gd2/*", GLOB_ONLYDIR);
if (count($matches) == 1 && $matches[0] == "gd2/subdir") { echo "ok"; }
unlink("gd2/file.txt");
rmdir("gd2/subdir");
rmdir("gd2");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: php-verified `glob($pattern, GLOB_MARK)` appends `/` to directory
/// matches only (files matched by the same pattern keep no trailing slash).
#[test]
fn test_glob_mark_appends_slash_to_directories_only() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("gd3");
mkdir("gd3/subdir");
file_put_contents("gd3/file.txt", "x");
$matches = glob("gd3/*", GLOB_MARK);
sort($matches);
echo implode(",", $matches);
unlink("gd3/file.txt");
rmdir("gd3/subdir");
rmdir("gd3");
"#,
    );
    assert_eq!(out, "gd3/file.txt,gd3/subdir/");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: php-verified `glob($pattern, GLOB_NOSORT | GLOB_ONLYDIR)` combines two
/// flags via bitwise OR (constant-folded to a single literal by the time EIR
/// codegen validates it) and still only returns directory matches.
#[test]
fn test_glob_combined_flags() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
mkdir("gd4");
mkdir("gd4/subdir");
file_put_contents("gd4/file.txt", "x");
$matches = glob("gd4/*", GLOB_NOSORT | GLOB_ONLYDIR);
if (count($matches) == 1 && $matches[0] == "gd4/subdir") { echo "ok"; }
unlink("gd4/file.txt");
rmdir("gd4/subdir");
rmdir("gd4");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// H5: `glob()` with no matches returns an empty array, not `false`
/// (php-verified: `var_dump(glob("nomatch*"))` → `array(0) {}`).
#[test]
fn test_glob_no_match_returns_empty_array() {
    let (out, dir) = compile_and_run_in_dir_ir(
        r#"<?php
$r = glob("definitely_does_not_exist_xyz*");
if (is_array($r) && count($r) === 0) { echo "ok"; }
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies tempnam creates a unique file in the given directory and that it
/// exists immediately, then cleans up the temporary file.
#[test]
fn test_glob_stream_wrapper_iterates_matches() {
    // Phase 6: opendir("glob://pattern") returns a synthetic directory
    // resource backed by libc glob; readdir iterates the matches, closedir
    // releases the gl_pathv, rewinddir restarts the iteration. libc glob
    // returns the matches in sorted order on every target we support.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
mkdir("gw");
file_put_contents("gw/a.txt", "1");
file_put_contents("gw/b.txt", "2");
$h = opendir("glob://gw/*.txt");
$first = readdir($h);
$second = readdir($h);
$end = readdir($h);
rewinddir($h);
$first_again = readdir($h);
closedir($h);
echo $first . "|" . $second . "|" . ($end === false ? "end" : "x") . "|" . $first_again;
unlink("gw/a.txt");
unlink("gw/b.txt");
rmdir("gw");
"#,
    );
    assert_eq!(out, "gw/a.txt|gw/b.txt|end|gw/a.txt");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for tempnam.
#[test]
fn test_tempnam() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$tmp = tempnam(".", "test");
if (file_exists($tmp)) { echo "ok"; }
unlink($tmp);
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for disk space positive and ordered.
#[test]
fn test_disk_space_positive_and_ordered() {
    let out = compile_and_run(
        r#"<?php
$free = disk_free_space("/");
$total = disk_total_space("/");
echo $free > 0 ? "f" : "F";
echo $total > 0 ? "t" : "T";
echo $total >= $free ? "o" : "O";
"#,
    );
    assert_eq!(out, "fto");
}

/// Verifies compiled PHP output for disk free space invalid path returns zero.
#[test]
fn test_disk_free_space_invalid_path_returns_zero() {
    let out = compile_and_run(r#"<?php var_dump(disk_free_space("/no/such/path/xyz123"));"#);
    assert_eq!(out, "float(0)\n");
}
