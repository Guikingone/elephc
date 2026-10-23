//! Purpose:
//! Integration tests for arrays and hashes of buffer handles (task fdc821c1): literals, appends,
//! element reads, foreach, `array_push`/`array_pop`/`array_shift`, variadic parameters and typed
//! properties, with every handle stored as one raw word next to the pointer-word arms.
//!
//! Called from:
//! - `cargo test --test codegen_tests test_buffer_array` through Rust's test harness.
//!
//! Key details:
//! - Buffer handles are not refcounted and are never null: a missing element or key and a pop of
//!   an empty array read as handle 0, which every buffer operation rejects with the
//!   use-after-free diagnostic.
//! - Scalar, width and packed-record buffers are covered, since the element access that follows
//!   an array read must keep the buffer's own element type.

use crate::support::*;

/// Buffer factory shared by the tests: `filled(3, 10)` holds 10, 11, 12.
const PRELUDE: &str = r#"<?php
function filled(int $n, int $start): buffer<int> {
    $b = buffer_new<int>($n);
    for ($i = 0; $i < $n; $i++) { $b[$i] = $start + $i; }
    return $b;
}
"#;

/// Verifies an array literal of buffers keeps each handle: element reads reach the buffers'
/// elements and lengths, foreach binds each handle, and a write through a read element lands in
/// the shared buffer.
#[test]
fn test_buffer_array_literals_read_elements_and_iterate() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$a = filled(3, 10);
$b = filled(2, 20);
$list = [$a, $b];
echo count($list), "|", $list[0][2], "|", $list[1][0], "|", buffer_len($list[1]), "\n";
foreach ($list as $i => $buf) { echo $i, ":", $buf[0], "+", buffer_len($buf), " "; }
echo "\n";
$second = $list[1];
$second[1] = 99;
echo $b[1], "\n";
"#
    ));
    assert_eq!(out, "2|12|20|2\n0:10+3 1:20+2 \n99\n");
}

/// Verifies appends, index writes, `array_push()`, `array_pop()` and `array_shift()` on an array
/// of buffers keep the handles, and the popped and shifted values are usable buffers.
#[test]
fn test_buffer_array_appends_index_writes_and_builtins() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$list = [];
$list[] = filled(1, 1);
$list[] = filled(1, 2);
$list[0] = filled(1, 3);
array_push($list, filled(1, 4), filled(1, 5));
echo count($list), "|", $list[0][0], "|", $list[1][0], "|", $list[3][0], "\n";
$last = array_pop($list);
$first = array_shift($list);
echo $last[0], "|", $first[0], "|", count($list), "|", $list[0][0], "\n";
"#
    ));
    assert_eq!(out, "4|3|2|5\n5|3|2|2\n");
}

/// Verifies a variadic `buffer<T> ...$bufs` parameter collects its handles into an array that
/// foreach and element reads consume, for positional arguments, a spread, and no arguments.
#[test]
fn test_buffer_array_variadics_and_spreads() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"function total(buffer<int> ...$bufs): int {
    $sum = 0;
    foreach ($bufs as $buf) {
        for ($i = 0; $i < buffer_len($buf); $i++) { $sum += $buf[$i]; }
    }
    return $sum;
}
$a = filled(3, 1);
$b = filled(2, 10);
echo total($a, $b), "|", total(...[$a, $b, $a]), "|", total(), "\n";
"#
    ));
    assert_eq!(out, "27|33|0\n");
}

/// Verifies arrays of width buffers and of packed-record buffers keep their element types: a
/// `buffer<u16>` element still truncates and a `buffer<Vec2>` element still reads records.
#[test]
fn test_buffer_array_of_width_and_record_buffers() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public f32 $x; public f32 $y; }
buffer<u16> $idx = buffer_new<u16>(3);
$idx[2] = 70000;
buffer<Vec2> $pts = buffer_new<Vec2>(2);
$pts[1]->x = 1.5;
$indices = [$idx];
$meshes = [$pts, $pts];
echo $indices[0][2], "|", $meshes[1][1]->x, "\n";
foreach ($meshes as $m) { $r = $m[1]; echo $r->x, " "; }
echo "\n";
"#,
    );
    assert_eq!(out, "4464|1.5\n1.5 1.5 \n");
}

/// Verifies a hash of buffers stores the handles under keys, accepts key writes, and binds the
/// handle in a keyed foreach.
#[test]
fn test_buffer_hashes_store_and_iterate() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$a = filled(2, 5);
$b = filled(1, 8);
$h = ["a" => $a, "b" => $b];
$h["c"] = filled(1, 9);
echo $h["b"][0], "|", $h["c"][0], "|", count($h), "\n";
foreach ($h as $k => $buf) { echo $k, "=", $buf[0], " "; }
echo "\n";
"#
    ));
    assert_eq!(out, "8|9|3\na=5 b=8 c=9 \n");
}

/// Verifies a typed `array<buffer<T>>` property starts from its empty default, takes appends
/// and `array_push()`, and reads back through the property.
#[test]
fn test_buffer_array_in_a_typed_property() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"class Pool {
    public array<buffer<int> > $pages = [];
    public function add(buffer<int> $page): void { $this->pages[] = $page; }
}
$pool = new Pool();
$pool->add(filled(2, 30));
array_push($pool->pages, filled(1, 40));
echo count($pool->pages), "|", $pool->pages[0][1], "|", $pool->pages[1][0], "\n";
foreach ($pool->pages as $page) { echo buffer_len($page), " "; }
echo "\n";
"#
    ));
    assert_eq!(out, "2|31|40\n2 1 \n");
}

/// Verifies a missing element of a buffer array reads as handle 0, so using it reports the
/// use-after-free diagnostic instead of touching memory.
#[test]
fn test_buffer_array_missing_element_is_an_invalid_handle() {
    let err = compile_and_run_expect_failure(&format!(
        "{PRELUDE}{}",
        r#"$list = [filled(1, 1)];
$miss = $list[3];
echo buffer_len($miss);
"#
    ));
    assert!(err.contains("use of buffer after buffer_free()"), "{}", err);
}

/// Verifies `array_pop()` of an emptied buffer array hands back handle 0, which the next buffer
/// operation rejects with the use-after-free diagnostic.
#[test]
fn test_buffer_array_pop_of_an_empty_array_is_an_invalid_handle() {
    let err = compile_and_run_expect_failure(&format!(
        "{PRELUDE}{}",
        r#"$list = [filled(1, 1)];
$only = array_pop($list);
echo $only[0], "\n";
$none = array_pop($list);
echo buffer_len($none);
"#
    ));
    assert!(err.contains("use of buffer after buffer_free()"), "{}", err);
}
