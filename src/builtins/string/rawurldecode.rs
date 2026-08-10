//! Purpose:
//! Home of the PHP `rawurldecode` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The typed runtime target has a validated `Str -> Str` EIR signature.
//! - Its fresh result ownership replaces the historical independent-storage flag.

use crate::ir::{RuntimeCallTarget, UnaryStringRuntime};

builtin! {
    contract: "rawurldecode",
    semantics: crate::builtins::semantics::unary_string_runtime(
        RuntimeCallTarget::UnaryString(UnaryStringRuntime::RawUrlDecode),
        crate::ir::Effects::PURE,
    ),
}
