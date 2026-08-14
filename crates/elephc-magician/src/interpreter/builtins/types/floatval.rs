//! Purpose:
//! Joins Magician to the shared runtime builtin contract for `floatval`.
//!
//! Called from:
//! - `crate::interpreter::builtins::registry` inventory assembly.
//!
//! Key details:
//! - Behavior dispatches by `RuntimeBuiltinId` through the versioned
//!   generated-runtime boxed-cell ABI; no Magician algorithm lives here.

eval_builtin! {
    contract: "floatval",
    area: Types,
    direct: none,
    values: none,
}
