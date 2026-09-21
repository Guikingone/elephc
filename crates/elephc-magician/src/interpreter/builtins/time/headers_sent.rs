//! Purpose:
//! Joins Magician to the shared runtime builtin contract for `headers_sent`.
//!
//! Called from:
//! - `crate::interpreter::builtins::registry` inventory assembly.
//!
//! Key details:
//! - Behavior dispatches by `RuntimeBuiltinId` through the versioned generated-runtime
//!   boxed-cell ABI; no Magician algorithm lives here. The flag is process state the compiled
//!   side owns, so answering it locally would report "not sent" for output the compiled program
//!   had already flushed.
//! - Only the no-argument form is joined. The two by-reference out-parameters PHP also accepts
//!   need out-parameter transport the version-one boxed-cell ABI does not carry.

eval_builtin! {
    contract: "headers_sent",
    area: Time,
    direct: none,
    values: none,
}
