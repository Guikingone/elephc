//! Purpose:
//! Walks class properties, methods, and constants during magic-constant substitution.
//! Applies expression and statement walkers to defaults, bodies, promoted-property assignments,
//! and class-constant initializers.
//!
//! Called from:
//! - `crate::magic_constants::walker::stmts` and trait binding passes.
//!
//! Key details:
//! - Member traversal preserves declaration metadata while updating only magic-constant-bearing children.
//! - Class-constant initializers must be walked with the enclosing class/trait scope active so
//!   `__CLASS__`, `__TRAIT__`, `__FUNCTION__`, and `__METHOD__` lower before type inference.

use crate::parser::ast::{ClassConst, ClassMethod, ClassProperty};

use super::exprs::walk_expr;
use super::stmts::walk_program;
use super::Pass;

/// Walks a class property, applying `pass` to its default-value expression if present.
///
/// - `prop`: The class property to walk.
/// - `pass`: The pass (visitor) to apply to child expressions.
///
/// Returns a new `ClassProperty` with the default expression replaced by the result
/// of walking it, or the original default if none existed. Other fields are preserved unchanged.
pub(in crate::magic_constants) fn walk_class_property<P: Pass>(
    prop: ClassProperty,
    pass: &mut P,
) -> ClassProperty {
    ClassProperty {
        default: prop.default.map(|e| walk_expr(e, pass)),
        ..prop
    }
}

/// Walks a class constant, applying `pass` to its initializer expression.
///
/// Class-constant initializers are otherwise skipped by the statement walker, so magic
/// constants inside `const X = <expr>;` would reach type inference un-lowered and panic.
/// This routes the initializer through the active pass with whatever class/trait/namespace
/// scope the caller has entered, matching the method-body substitution.
///
/// - `constant`: The class constant to walk.
/// - `pass`: The pass (visitor) to apply to the initializer expression.
///
/// Returns a new `ClassConst` with only its `value` replaced by the walked expression;
/// name, visibility, `is_final`, span, and attributes are preserved unchanged.
pub(in crate::magic_constants) fn walk_class_constant<P: Pass>(
    constant: ClassConst,
    pass: &mut P,
) -> ClassConst {
    ClassConst {
        value: walk_expr(constant.value, pass),
        ..constant
    }
}

/// Walks a class method, applying `pass` to parameter defaults and the method body.
///
/// Calls `pass.enter_method` before walking and `pass.leave_method` after, so the pass
/// can track method entry/exit for context (e.g., `__METHOD__` constant).
///
/// - `method`: The class method to walk.
/// - `pass`: The pass (visitor) to apply to expressions and statements.
///
/// Returns a new `ClassMethod` with defaults and body walked; declaration metadata (name,
/// visibility, static, etc.) is preserved unchanged.
pub(in crate::magic_constants) fn walk_class_method<P: Pass>(
    method: ClassMethod,
    pass: &mut P,
) -> ClassMethod {
    pass.enter_method(&method.name);
    let new_params = method
        .params
        .into_iter()
        .map(|(n, t, default, by_ref)| (n, t, default.map(|d| walk_expr(d, pass)), by_ref))
        .collect();
    let new_body = walk_program(method.body, pass);
    pass.leave_method();
    ClassMethod {
        params: new_params,
        body: new_body,
        ..method
    }
}
