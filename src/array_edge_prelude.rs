//! Purpose:
//! Declares PHP 8.5's `array_first()` and `array_last()` as AST-built functions.
//!
//! Called from:
//! - `crate::pipeline::compile()` in the compat-prelude phase, after autoload expansion, so a
//!   call only an autoloaded class makes is seen, and the codegen test harness.
//!
//! Key details:
//! - Both answer `null` for an empty array and otherwise the value at the first/last position
//!   in iteration order, never touching the array's internal pointer — php 8.5's contract.
//! - They matter beyond direct callers: `symfony/polyfill-php85`'s bootstrap returns early on
//!   an 8.5 profile, so code generated against php 8.5 (a Symfony container dumping
//!   `\array_first(...)`) has no other definition.
//! - Pay-for-use: injected only when the program names one of the functions, and never over a
//!   declaration of the same name the program makes itself.
//! - The interpreter has its own implementations
//!   (`elephc_magician::interpreter::builtins::array::array_first` / `array_last`), because a
//!   compiler prelude is never reachable from interpreted code.

use crate::parser::ast::{BinOp, Program, Stmt, StmtKind};
use crate::synthetic_class::{
    e_array, e_binop, e_call, e_index, e_null, e_var, function, internal_declarations, s_foreach,
    s_if, s_return, t_array, t_mixed,
};

/// Prelude inventory group holding the declarations.
pub(crate) const GROUP: &str = "array_edge";

/// The PHP functions this prelude declares.
pub(crate) const FUNCTIONS: &[&str] = &["array_first", "array_last"];

/// Builds the two functions.
fn function_declarations() -> Vec<Stmt> {
    // function array_first(array $array): mixed {
    //     foreach ($array as $value) { return $value; }
    //     return null;
    // }
    let first = function("array_first")
        .param("array", t_array())
        .returns(t_mixed())
        .body(vec![
            s_foreach(e_var("array"), None, "value", vec![s_return(e_var("value"))]),
            s_return(e_null()),
        ])
        .build();

    // function array_last(array $array): mixed {
    //     if ($array === []) { return null; }
    //     return $array[array_key_last($array)];
    // }
    let last = function("array_last")
        .param("array", t_array())
        .returns(t_mixed())
        .body(vec![
            s_if(
                e_binop(e_var("array"), BinOp::StrictEq, e_array(Vec::new())),
                vec![s_return(e_null())],
                Vec::new(),
                None,
            ),
            s_return(e_index(
                e_var("array"),
                e_call("array_key_last", vec![e_var("array")]),
            )),
        ])
        .build();

    vec![first, last]
}

/// Returns the declarations to inject.
pub(crate) fn declarations() -> Program {
    internal_declarations(function_declarations)
}

/// Returns whether a top-level, namespace-less, guarded or synthetic statement declares the
/// function `name`.
fn program_declares_function(program: &[Stmt], name: &str) -> bool {
    program.iter().any(|stmt| match &stmt.kind {
        StmtKind::FunctionDecl { name: declared, .. } => declared.eq_ignore_ascii_case(name),
        StmtKind::NamespaceBlock { name: None, body }
        | StmtKind::Synthetic(body)
        | StmtKind::IncludeOnceGuard { body, .. } => program_declares_function(body, name),
        _ => false,
    })
}

/// Prepends the declarations when the program names one of the functions and declares none of
/// them itself.
pub fn inject_if_used(
    program: Program,
    inventory: &mut crate::optimize::reachability::PreludeInventory,
) -> Program {
    let used = crate::prelude_prune::usage::collect(&program);
    let names_function = FUNCTIONS.iter().any(|name| {
        crate::opcache_prelude::detect::program_references(&program, name)
            || used.literals.contains(&crate::names::php_symbol_key(name))
    });
    if !names_function
        || FUNCTIONS
            .iter()
            .any(|name| program_declares_function(&program, name))
    {
        return program;
    }
    let mut combined = declarations();
    inventory.record_program(GROUP, &combined);
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The surface is exactly the two functions.
    #[test]
    fn declares_array_first_and_array_last() {
        let functions: Vec<String> = declarations()
            .into_iter()
            .filter_map(|stmt| match stmt.kind {
                StmtKind::FunctionDecl { name, .. } => Some(name),
                _ => None,
            })
            .collect();
        assert_eq!(functions, FUNCTIONS);
    }
}
