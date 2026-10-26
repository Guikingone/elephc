//! Purpose:
//! Runs trusted compiler AST builders through the complete native fixture pipeline.
//!
//! Called from:
//! - Compiler-internal expression regression tests and the fixture compiler.
//!
//! Key details:
//! - Per-thread state isolates parallel tests; the previous builder is restored on panic.
//! - Transformations run before frontend rewrites, resolution, checking and EIR lowering.

use elephc::parser::ast::Program;
use std::cell::Cell;

type Transform = fn(&mut Program);
thread_local! {
    static TRANSFORM: Cell<Option<Transform>> = const { Cell::new(None) };
}

/// Applies the active trusted builder to one freshly parsed fixture.
pub(crate) fn transform_fixture_ast(program: &mut Program) {
    if let Some(transform) = TRANSFORM.with(Cell::get) { transform(program); }
}

/// Scopes a compiler builder to this thread and restores it even when the test panics.
pub(crate) fn with_fixture_ast_transform<T>(transform: Transform, body: impl FnOnce() -> T) -> T {
    struct Restore(Option<Transform>);
    impl Drop for Restore {
        fn drop(&mut self) { TRANSFORM.with(|state| state.set(self.0)); }
    }
    let _restore = Restore(TRANSFORM.with(|state| state.replace(Some(transform))));
    body()
}
