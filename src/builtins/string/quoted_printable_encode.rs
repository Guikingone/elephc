//! Purpose:
//! Home of the PHP `quoted_printable_encode` builtin and its backend-neutral runtime semantics.
//!
//! Called from:
//! - The builtin registry, checker, optimizer, and AST-to-EIR builtin lowering path.
//!
//! Key details:
//! - The typed runtime target has a validated `Str -> Str` EIR signature.
//! - Encoding only inspects its argument's bytes, so the call is `PURE`.
//! - The MIME quoted-printable rules live in `__rt_quoted_printable_encode`: control bytes,
//!   `0x7F`, high-bit bytes, `=`, and a space directly before a `CR` become `=XX`; an embedded
//!   `CRLF` is copied through; lines are folded at 75 columns with a trailing `=`.
//! - Concrete helper symbols and registers are selected only by the target backend.

use crate::ir::{RuntimeCallTarget, UnaryStringRuntime};

builtin! {
    contract: "quoted_printable_encode",
    semantics: crate::builtins::semantics::unary_string_runtime(
        RuntimeCallTarget::UnaryString(UnaryStringRuntime::QuotedPrintableEncode),
        crate::ir::Effects::PURE,
    ),
}
