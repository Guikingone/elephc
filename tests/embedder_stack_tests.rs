//! Purpose:
//! Pins that the individually guarded compiler phases survive `MAX_COMPILER_NESTING` on a SMALL
//! thread stack, with no `compiler_stack::with_compiler_stack` wrapper anywhere above them.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - This is the embedder that calls one phase at a time, and only these fixtures can tell
//!   whether that still works: every other in-process test drives the WHOLE pipeline through a
//!   harness that wraps itself in `with_compiler_stack`, so it passes whether or not the phases
//!   carry their own budget (issue #686).
//! - The stack size is set HERE rather than inherited, so `RUST_MIN_STACK` — which the codegen
//!   suite raises to 32 MiB — cannot hide a missing guard. 256 KiB is far below what 1024
//!   nesting levels need from the passes below the parser.
//! - A regression aborts the test PROCESS with `has overflowed its stack` rather than failing an
//!   assertion. That is the failure mode worth pinning, and the reason these live in their own
//!   binary: an abort takes the whole test process with it.

use std::collections::HashSet;

/// The stack an embedder's worker thread plausibly has. Well under what depth 1024 needs.
const EMBEDDER_STACK_BYTES: usize = 256 * 1024;

/// Source nesting at the compiler's own documented limit.
const NESTING_DEPTH: usize = 1024;

/// Runs `body` on a thread with [`EMBEDDER_STACK_BYTES`] of stack and returns its answer.
fn on_a_small_embedder_stack<R: Send + 'static>(
    body: impl FnOnce() -> R + Send + 'static,
) -> R {
    std::thread::Builder::new()
        .name("embedder-small-stack".to_string())
        .stack_size(EMBEDDER_STACK_BYTES)
        .spawn(body)
        .expect("spawning the small embedder stack")
        .join()
        .expect("the embedder thread panicked")
}

/// `$a = [[[…1…]]]` at the compiler's nesting limit.
fn deeply_nested_source() -> String {
    format!(
        "<?php\n$a = {}1{};\necho count($a);\n",
        "[".repeat(NESTING_DEPTH),
        "]".repeat(NESTING_DEPTH)
    )
}

/// Parses the fixture, which every phase below starts from.
fn parse_deeply_nested() -> elephc::parser::ast::Program {
    let source = deeply_nested_source();
    let tokens = elephc::lexer::tokenize(&source).expect("tokenize");
    elephc::parser::parse(&tokens).expect("parse")
}

/// Verifies `parser::parse` walks its own limit on a small stack, with brackets and with
/// parentheses.
///
/// A GUARD rather than a reproduction, and deliberately so: measured, the parser is the
/// shallowest of these walkers and survives 1024 levels of either shape on this stack with its
/// wrapper removed. That is consistent with the issue itself — the aborts were always in the
/// passes below. What this pins is that the entry still carries the budget, so the parser does
/// not become the outlier the day its frames grow.
#[test]
fn parsing_survives_the_nesting_limit_on_a_small_embedder_stack() {
    let bracketed = on_a_small_embedder_stack(parse_deeply_nested);
    assert_eq!(bracketed.len(), 2);
    let parenthesized = on_a_small_embedder_stack(|| {
        let source = format!(
            "<?php\n$a = {}1{};\necho $a;\n",
            "(".repeat(NESTING_DEPTH),
            ")".repeat(NESTING_DEPTH)
        );
        let tokens = elephc::lexer::tokenize(&source).expect("tokenize");
        elephc::parser::parse(&tokens).expect("parse")
    });
    assert_eq!(parenthesized.len(), 2);
}

/// Verifies the magic-constant walker survives the same depth called on its own.
#[test]
fn magic_constant_substitution_survives_the_nesting_limit_on_a_small_embedder_stack() {
    let program = on_a_small_embedder_stack(|| {
        let ast = parse_deeply_nested();
        elephc::magic_constants::substitute_file_and_scope_constants(
            ast,
            std::path::Path::new("embedder.php"),
        )
    });
    assert_eq!(program.len(), 2);
}

/// Verifies the constant folder survives the same depth called on its own.
#[test]
fn constant_folding_survives_the_nesting_limit_on_a_small_embedder_stack() {
    let program = on_a_small_embedder_stack(|| {
        let ast = parse_deeply_nested();
        elephc::optimize::fold_constants(ast)
    });
    assert_eq!(program.len(), 2);
}

/// Verifies the type checker survives the same depth called on its own.
#[test]
fn type_checking_survives_the_nesting_limit_on_a_small_embedder_stack() {
    let result = on_a_small_embedder_stack(|| {
        let ast = parse_deeply_nested();
        let ast = elephc::conditional::apply(ast, &HashSet::new());
        let ast = elephc::name_resolver::resolve(ast).expect("name resolve");
        let ast = elephc::optimize::fold_constants(ast);
        elephc::types::check(&ast).map(|_| ()).map_err(|e| e.message)
    });
    assert_eq!(result, Ok(()));
}

/// Verifies constant propagation survives the same depth called on its own.
///
/// This is the pass the first cut of the fix missed: it runs AFTER the checker, so a fixture
/// that stopped at type checking could not reach it, and a full compile went through the
/// whole-run budget instead. Called directly, as an embedder running the optimizer would.
#[test]
fn constant_propagation_survives_the_nesting_limit_on_a_small_embedder_stack() {
    let program = on_a_small_embedder_stack(|| {
        let ast = parse_deeply_nested();
        let ast = elephc::conditional::apply(ast, &HashSet::new());
        let ast = elephc::name_resolver::resolve(ast).expect("name resolve");
        let ast = elephc::optimize::fold_constants(ast);
        let check = elephc::types::check(&ast).expect("type check");
        elephc::optimize::propagate_constants(ast, check.mixed_storage_local_names())
    });
    assert_eq!(program.len(), 2);
}
