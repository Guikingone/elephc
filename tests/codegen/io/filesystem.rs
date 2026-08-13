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

/// Verifies sys_get_temp_dir returns a usable absolute directory.
///
/// This used to require the answer to CONTAIN "tmp", which is not true of php: on macOS it
/// hands out a private per-user directory such as `/var/folders/xc/…/T`, with no "tmp" in it
/// anywhere. The assertion only held because elephc answered a hardcoded `/tmp`, so the test
/// pinned the divergence rather than the behaviour.
///
/// What the answer must satisfy on every platform is checked instead; the relationship to
/// `TMPDIR` is pinned separately by `test_sys_get_temp_dir_follows_tmpdir`.
#[test]
fn test_sys_get_temp_dir() {
    let out = compile_and_run(
        r#"<?php
$tmp = sys_get_temp_dir();
echo var_export($tmp !== "" && is_dir($tmp), true);
"#,
    );
    assert_eq!(out, "true");
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

/// Verifies `scandir()` on a missing directory answers FALSE, as php does.
///
/// Three eras of this test: AArch64 first handed `opendir()`'s NULL straight to `readdir()`
/// and crashed; then the empty listing papered over the crash while diverging from php's
/// `false`; now the union is real. `=== false` is the manual's own failure test, and
/// `count()` on the false raises php's TypeError — both asserted, because the empty-array era
/// made exactly those two observations impossible.
#[test]
fn test_scandir_on_a_missing_directory_answers_false() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$entries = @scandir("no_such_directory_here");
var_dump($entries === false);
try {
    count($entries);
    echo "uncaught";
} catch (TypeError $e) {
    echo "count: ", $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "bool(true)\ncount: count(): Argument #1 ($value) must be of type Countable|array, false given"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `scandir()` reports an unopenable directory the way php does — and stays SILENT
/// when the directory opens.
///
/// php-src writes TWO lines for one failure, the second naming the error number, and elephc
/// wrote neither: the failure was completely mute, so a typo'd path produced an empty listing
/// and no clue. Neither line needed a composer of its own, because `__rt_errno_warning` already
/// appends `strerror` and the newline and so serves as the tail of both.
///
/// The successful call at the end is not padding. The failure block was first placed after the
/// `closedir` that ends the read loop, so the SUCCESS path fell straight into it and every
/// working `scandir()` printed a warning carrying a stale errno. Only a probe that exercised a
/// directory which opens could catch that, and the first one did.
///
/// `@` suppression and the 3000-iteration loop are asserted together: the error number is
/// rendered by `__rt_itoa`, which formats into the shared 64 KiB concat arena and advances its
/// cursor, so a loop over unreadable paths would eat the buffer if the diagnostic did not hand
/// its scratch back.
#[test]
fn test_scandir_reports_an_unopenable_directory_like_php() {
    let out = compile_and_run_capture(
        r#"<?php
scandir("/pas/la");
scandir("/etc/hosts");
@scandir("/pas/la");
for ($i = 0; $i < 3000; $i++) {
    @scandir("/pas/la/deep/path/number/$i");
}
$here = scandir(".");
echo "opened=", (count($here) > 0 ? "yes" : "no"), "\n";
"#,
    );
    assert!(out.success, "the diagnostics must not disturb the program");
    assert_eq!(out.stdout, "opened=yes\n");
    assert_eq!(
        out.stderr,
        "Warning: scandir(/pas/la): Failed to open directory: No such file or directory\n\
         Warning: scandir(): (errno 2): No such file or directory\n\
         Warning: scandir(/etc/hosts): Failed to open directory: Not a directory\n\
         Warning: scandir(): (errno 20): Not a directory\n",
        "both lines, both error numbers, nothing from the suppressed calls, \
         and nothing at all from the directory that opened"
    );
}

/// Verifies `file_put_contents()` on an unopenable path warns and answers false — and writes
/// the payload NOWHERE.
///
/// The open result was never checked. On macOS a failed open answers the ERRNO with the carry
/// set, so the payload was written through descriptor 2 — the caller's own stderr — and the
/// byte count reported SUCCESS: `file_put_contents("/no/such/dir/x", $secret)` leaked the
/// secret to the terminal and returned int(7). php warns and answers false, which is also why
/// the declaration is now `int|false` rather than `Int`: with `Int`, the manual's own
/// `=== false` failure test could never fire.
///
/// The stdout assertion is exact so a payload leaking to EITHER stream fails the test.
#[test]
fn test_file_put_contents_on_an_unopenable_path_answers_false() {
    let out = compile_and_run_capture(
        r#"<?php
$n = file_put_contents("/no/such/dir/leak.txt", "SECRET-PAYLOAD");
var_dump($n);
var_dump($n === false);
var_dump(@file_put_contents("/no/such/dir/leak.txt", "SECRET-PAYLOAD"));
"#,
    );
    assert!(out.success, "a failed write is not a crash");
    assert_eq!(out.stdout, "bool(false)\nbool(true)\nbool(false)\n");
    assert_eq!(
        out.stderr,
        "Warning: file_put_contents(/no/such/dir/leak.txt): Failed to open stream: \
         No such file or directory\n",
        "one warning in php's wording; the @-suppressed call prints nothing, \
         and the payload appears on neither stream"
    );
}

/// Verifies `scandir()` sorts like php and its `array|false` union flows through the family.
///
/// php sorts ascending by default, descending for SCANDIR_SORT_DESCENDING, and keeps readdir
/// order only for SCANDIR_SORT_NONE — elephc answered readdir order for every call, which is
/// filesystem-dependent. The file names are created in REVERSE alphabetical order so an
/// unsorted listing cannot pass by accident.
///
/// The consumers each pin one leg of the union machinery: `in_array`/`array_values`/
/// `array_map`/`array_filter`/`array_search` go through the argument unbox (which must borrow,
/// not own — an owned unbox freed the listing UNDER the box and a later `sort($d)` sorted
/// freed memory), and `sort($d)` goes through the in-place path where the box must remain the
/// listing's sole owner or the copy-on-write split sorts a copy.
#[test]
fn test_scandir_sorts_like_php_and_the_union_flows_through_the_family() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
mkdir("sd");
file_put_contents("sd/z.txt", "1");
file_put_contents("sd/a.txt", "2");
echo implode(",", scandir("sd")), "\n";
echo implode(",", scandir("sd", SCANDIR_SORT_DESCENDING)), "\n";
$none = scandir("sd", SCANDIR_SORT_NONE);
sort($none);
echo implode(",", $none), "\n";
$d = scandir("sd");
echo "in=", var_export(in_array("a.txt", $d), true), "\n";
sort($d);
echo "s0=", $d[2], "\n";
echo "vals=", count(array_values(scandir("sd"))), "\n";
$up = array_map(fn($f) => strtoupper($f), scandir("sd"));
echo "map=", $up[2], "\n";
echo "search=", var_export(array_search("z.txt", scandir("sd")), true), "\n";
echo "filter=", count(array_filter(scandir("sd"), fn($f) => $f !== ".")), "\n";
unlink("sd/z.txt"); unlink("sd/a.txt"); rmdir("sd");
"#,
    );
    assert_eq!(
        out,
        ".,..,a.txt,z.txt\nz.txt,a.txt,..,.\n.,..,a.txt,z.txt\nin=true\ns0=a.txt\nvals=4\nmap=A.TXT\nsearch=3\nfilter=3\n"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a runtime `false` flowing into an array-taking builtin raises php's TypeError.
///
/// The message is composed at compile time from php's own parameter naming — measured, not
/// derived — and the throw happens at the argument, before the consumer's lowering ever sees
/// the value. `sort($d)` exercises the by-reference spelling of the same contract.
#[test]
fn test_an_array_or_false_union_argument_throws_phps_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
$d = @scandir("/no/such/dir");
try {
    in_array("x", $d);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    sort($d);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
echo "alive\n";
"#,
    );
    assert!(out.success);
    assert_eq!(
        out.stdout,
        "in_array(): Argument #2 ($haystack) must be of type array, false given\n\
         sort(): Argument #1 ($array) must be of type array, false given\nalive\n"
    );
}

/// Verifies `array_reverse()` on a STRING array — literal and through the scandir union.
///
/// String slots are 16-byte (ptr, len) descriptors, and the shared 8-byte gate refused them
/// since it existed: `array_reverse(["a","b"])` failed to compile on plain literals. The
/// string variant re-persists each element into the new array, so the result owns its bytes
/// and the source's lifetime asks no aliasing questions.
#[test]
fn test_array_reverse_on_a_string_array_matches_php() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
echo implode(",", array_reverse(["a", "b", "c"])), "|";
mkdir("rvd");
file_put_contents("rvd/a.txt", "1");
file_put_contents("rvd/z.txt", "2");
echo implode(",", array_reverse(scandir("rvd")));
unlink("rvd/a.txt"); unlink("rvd/z.txt"); rmdir("rvd");
"#,
    );
    assert_eq!(out, "c,b,a|z.txt,a.txt,..,.");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `array_diff()` and `array_intersect()` on STRING arrays.
///
/// Both refused string arrays at the shared 8-byte gate — plain literals included — which is
/// what kept `array_diff(scandir($d), [".", ".."])`, the most ordinary directory idiom in
/// PHP, from compiling. One parameterised string helper serves both operations (the loop is
/// identical, only the keep-on-match sense differs), comparing through `__rt_str_eq` and
/// re-persisting survivors.
///
/// The assertions are VALUE-based (implode) on purpose: the whole set-operation family
/// reindexes its result where php preserves the source keys, a pre-existing divergence the
/// string variants share with the Int helpers and which is tracked separately.
#[test]
fn test_string_array_set_operations_keep_the_right_values()
{
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
echo implode(",", array_diff(["a", "b", "c"], ["b"])), "|";
echo implode(",", array_intersect(["a", "b", "c"], ["b", "c", "z"])), "|";
mkdir("sod");
file_put_contents("sod/a.txt", "1");
file_put_contents("sod/b.txt", "2");
echo implode(",", array_diff(scandir("sod"), [".", ".."]));
unlink("sod/a.txt"); unlink("sod/b.txt"); rmdir("sod");
"#,
    );
    assert_eq!(out, "a,c|b,c|a.txt,b.txt");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `array_merge()` on STRING arrays, empty-literal mixes included.
///
/// php reindexes list keys on merge, which is exactly what two append loops produce — unlike
/// the set operations, there is no key divergence here. An empty literal carries a
/// `Never`-element type whose declared slot size is moot at length zero, so it rides along
/// with a string side rather than failing the one-common-layout rule.
#[test]
fn test_array_merge_on_string_arrays_matches_php() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
echo implode(",", array_merge(["a", "b"], ["c"])), "|";
echo implode(",", array_merge([], ["x", "y"])), "|";
mkdir("mgd");
file_put_contents("mgd/f.txt", "1");
echo implode(",", array_merge(scandir("mgd"), ["extra"]));
unlink("mgd/f.txt"); rmdir("mgd");
"#,
    );
    assert_eq!(out, "a,b,c|x,y|.,..,f.txt,extra");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies glob by creating two files matching a pattern, confirming both
/// are returned with their full paths, and cleaning up.
#[test]
fn test_glob_fn() {
    let (out, dir) = compile_and_run_in_dir(
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

/// A path that cannot be stat'd answers `false`, not a reading of zero.
///
/// The old expectation named the defect: `float(0)` is a legitimate reading for a full filesystem,
/// so `disk_free_space($d) === false` never fired and arithmetic on the result silently used zero.
/// PHP returns `float|false` and this is the `false`.
#[test]
fn test_disk_free_space_invalid_path_is_false() {
    let out = compile_and_run(r#"<?php var_dump(disk_free_space("/no/such/path/xyz123"));"#);
    assert_eq!(out, "bool(false)\n");
}
