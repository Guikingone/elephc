//! Purpose:
//! Parser coverage for retained generic declarations, nested type lists, and comparisons.
//!
//! Called from:
//! - `cargo test -p elephc-magician generics`.
//!
//! Key details:
//! - Right shifts split only while parsing type arguments.

use super::support::*;

/// Typed-local probing must not reinterpret reserved statement keywords as class types.
#[test]
fn generics_typed_local_keeps_statement_keyword_precedence() {
    for source in ["echo $value = 1;", "return $value = 1;"] {
        assert!(parse_fragment(source.as_bytes()).is_err(), "{source}");
    }
}

/// Native function/class/interface/trait declarations retain templates.
#[test]
fn generics_parse_declaration_families() {
    for source in [
        "function id<T>(T $v): T { return $v; }",
        "class Box<T> { public T $value; }",
        "interface Read<T> { public function get(): T; }",
        "trait Store<T> { public T $value; }",
    ] {
        let program = parse_fragment(source.as_bytes()).unwrap();
        assert!(matches!(&program.statements()[0], EvalStmt::GenericDecl(_)), "{source}");
    }
}

/// Nested generic closes coexist with ordinary PHP comparison and shift operators.
#[test]
fn generics_nested_lists_leave_comparisons_intact() {
    for source in ["new Box<Box<int>>(new Box<int>(1));", "echo id<array<array<int>>>([[1]])[0][0];", "echo $a < $b && $b > $c;", "echo 16 >> 2;"] {
        parse_fragment(source.as_bytes()).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
}

/// Malformed generic lists and invalid associative array keys fail during parsing.
#[test]
fn generics_malformed_lists_are_rejected() {
    for source in ["enum Choice<T> { case None; }", "function id<>(int $v) {}", "class Box<T, T> {}", "function id<T(T $v) {}", "function f(array<float, int> $v) {}"] {
        assert!(parse_fragment(source.as_bytes()).is_err(), "{source}");
    }
}
