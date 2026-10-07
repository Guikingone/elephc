//! Purpose:
//! Regresses generic type inference from static named argument unpacks.
//! Covers functions, methods, class construction, and constructor-local templates.
//!
//! Called from:
//! - `cargo test --test codegen_tests generics_named_spreads`.
//!
//! Key details:
//! - Inference uses the shared spread expansion without changing source evaluation order.
//! - PHPDoc inference remains active in strict PHP mode.

use crate::support::*;

/// Free functions infer from declaration positions after expanding named unpacks.
#[test]
fn test_generic_named_spreads_functions_and_duplicate_keys() {
    let source = r#"<?php
function box<T>(int $n, T $v): T { return $v; }
echo box(...["v" => "abc", "n" => 1]), "|",
    box(...["v" => 7, "v" => "last", "n" => 2]);
"#;
    assert_eq!(compile_and_run(source), "abc|last");
}

/// Instance and static methods share the same named-unpack inference as functions.
#[test]
fn test_generic_named_spreads_instance_and_static_methods() {
    let source = r#"<?php
class C {
    public function box<T>(int $n, T $v): T { return $v; }
    public static function copy<T>(int $n, T $v): T { return $v; }
}
echo (new C())->box(...["v" => "abc", "n" => 1]), "|",
    C::copy(...["v" => 7, "n" => 2]) + 1;
"#;
    assert_eq!(compile_and_run(source), "abc|8");
}

/// Generic classes, their static factories and constructor templates infer from named unpacks.
#[test]
fn test_generic_named_spreads_constructions_and_static_factories() {
    let source = r#"<?php
class Box<T> {
    public function __construct(int $n, public T $v) {}
    public static function of(int $n, T $v): Box<T> { return new Box($n, $v); }
}
class Value {
    public function __construct<T>(int $n, public T $v) {}
}
$box = new Box(...["v" => "abc", "n" => 1]);
$factory = Box::of(...["v" => 7, "n" => 2]);
$value = new Value(...["v" => "def", "n" => 3]);
echo $box->v, "|", $factory->v + 1, "|", $value->v;
"#;
    assert_eq!(compile_and_run(source), "abc|8|def");
}

/// Portable templates preserve unpack expression evaluation order in both CLI modes.
#[test]
fn test_generic_named_spreads_docblock_and_evaluation_order() {
    let source = r#"<?php
/**
 * @template T
 * @param T $v
 * @return T
 */
function box(int $n, $v) { return $v; }
function word(): string { echo "v"; return "abc"; }
function number(): int { echo "n"; return 1; }
echo box(...["v" => word(), "n" => number()]);
"#;
    for flags in [&[][..], &["--strict-php"][..]] {
        assert_eq!(compile_cli_file_and_run_with_flags(source, flags), "vnabc");
    }
}
