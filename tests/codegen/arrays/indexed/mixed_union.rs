//! Purpose:
//! Integration tests for the PHP array union operator `+` across mixed indexed/associative
//! array kinds, covering left-key precedence, numeric-string key normalization, empty operands,
//! heterogeneous payloads, and refcounted-value ownership through the promotion temporary.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Mixed-kind union promotes the indexed side to a hash via `__rt_array_to_hash`,
//!   then reuses the existing hash-union path; tests must validate left-key precedence
//!   and that the promoted temporary's refcounted/string payloads end up owned by the result.

use super::*;

#[test]
fn test_array_union_indexed_plus_assoc() {
    let out = compile_and_run(
        r#"<?php
$result = [1, 2, 3] + ["a" => 10];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "0=>1;1=>2;2=>3;a=>10;");
}

#[test]
fn test_array_union_assoc_plus_indexed_left_key_precedence() {
    let out = compile_and_run(
        r#"<?php
$result = ["a" => 1, 0 => "x"] + [10, 20, 30];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "a=>1;0=>x;1=>20;2=>30;");
}

#[test]
fn test_array_union_numeric_string_key_collision() {
    let out = compile_and_run(
        r#"<?php
$result = ["0" => "first"] + [10, 20];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "0=>first;1=>20;");
}

#[test]
fn test_array_union_empty_indexed_plus_assoc() {
    let out = compile_and_run(
        r#"<?php
$result = [] + ["a" => 1, "b" => 2];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "a=>1;b=>2;");
}

#[test]
fn test_array_union_assoc_plus_empty_indexed() {
    let out = compile_and_run(
        r#"<?php
$result = ["a" => 1, "b" => 2] + [];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "a=>1;b=>2;");
}

#[test]
fn test_array_union_heterogeneous_indexed_payload_plus_assoc() {
    let out = compile_and_run(
        r#"<?php
$result = [1, "two", 3.0, true] + ["x" => "y"];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "0=>1;1=>two;2=>3;3=>1;x=>y;");
}

#[test]
fn test_array_union_string_payload_indexed_plus_assoc() {
    let out = compile_and_run(
        r#"<?php
$result = ["hello", "world"] + ["greet" => "hi"];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "0=>hello;1=>world;greet=>hi;");
}

#[test]
fn test_array_union_assoc_string_value_keeps_left_on_int_key_collision() {
    let out = compile_and_run(
        r#"<?php
$result = [0 => "kept"] + [10, 20];
foreach ($result as $k => $v) {
    echo "$k=>$v;";
}
"#,
    );
    assert_eq!(out, "0=>kept;1=>20;");
}
