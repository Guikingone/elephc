//! Purpose:
//! Home of the PHP `constant` builtin: its single-source registry declaration and semantic
//! metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Literal names retain the referenced constant's precise type, while runtime names use the
//!   boxed `mixed` representation returned by the constant registry.
//! - An unknown name is a COMPILE error here, where reference PHP raises
//!   `Error: Undefined constant "X"` at runtime. A binary with no constant table cannot look the
//!   name up, and refusing at compile time is strictly more informative than a runtime fatal.
//! - Class constants and enum cases (`constant('Foo::BAR')`) are NOT supported: the name is
//!   resolved through the global constant table only.
//! - Literal lowering happens one level up, in
//!   `crate::ir_lower::expr::constants::lower_static_constant_call()`, which rewrites the call
//!   into the same EIR a bare `FOO` reference produces. Other names lower through the typed
//!   runtime-function path and the closed-world constant registry.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "constant",
    area: System,
    params: [name: Str],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Constant,
    ),
    summary: "Returns the value of a constant given its name.",
    php_manual: "https://www.php.net/manual/en/function.constant.php",
}

/// Returns a literal constant's precise type or `mixed` for a runtime name.
///
/// A leading `\` is stripped from literal names the way PHP's global-constant lookup does.
/// Runtime names are resolved by the emitted constant registry. Returns a `CompileError` for a
/// literal class-constant name or a literal unknown global constant.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    let literal = match &cx.args[0].kind {
        ExprKind::StringLiteral(name) => Some(name.clone()),
        ExprKind::NamedArg { name, value } if name == "name" => match &value.kind {
            ExprKind::StringLiteral(name) => Some(name.clone()),
            _ => None,
        },
        _ => None,
    };
    let Some(name) = literal else {
        return Ok(PhpType::Mixed);
    };
    let name = name.trim_start_matches('\\').to_string();
    if name.contains("::") {
        return Err(CompileError::new(
            cx.span,
            "constant() class constants are not supported; reference the constant directly",
        ));
    }
    match cx.checker.constants.get(&name) {
        Some(ty) => Ok(ty.clone()),
        None => Err(CompileError::new(
            cx.span,
            &format!("Undefined constant: {}", name),
        )),
    }
}
