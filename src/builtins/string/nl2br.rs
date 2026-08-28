//! Purpose:
//! Home of the PHP `nl2br` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The typed runtime target has a validated `Str -> Str` EIR signature.
//! - Concrete helper symbols and registers are selected only by the target backend.

use crate::ir::{RuntimeCallTarget, UnaryStringRuntime};

builtin! {
    contract: "nl2br",
    semantics: crate::builtins::semantics::unary_string_runtime(
        RuntimeCallTarget::UnaryString(UnaryStringRuntime::NlToBr),
        crate::ir::Effects::PURE,
    ),
}
