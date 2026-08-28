//! Purpose:
//! Home of the PHP `hex2bin` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The typed runtime target has a validated `Str -> Str` EIR signature.
//! - Invalid hexadecimal input retains its observable warning effect.

use crate::ir::{RuntimeCallTarget, UnaryStringRuntime};

builtin! {
    contract: "hex2bin",
    semantics: crate::builtins::semantics::unary_string_runtime(
        RuntimeCallTarget::UnaryString(UnaryStringRuntime::HexToBin),
        crate::ir::Effects::MAY_WARN,
    ),
}
