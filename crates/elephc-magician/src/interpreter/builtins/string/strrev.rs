//! Purpose:
//! Joins Magician to the shared runtime builtin contract for `strrev`.
//!
//! Called from:
//! - `crate::interpreter::builtins::registry` inventory assembly.
//!
//! Key details:
//! - Behavior dispatches by `RuntimeBuiltinId` through the versioned
//!   generated-runtime boxed-cell ABI; no Magician algorithm lives here.

eval_builtin! {
    contract: "strrev",
    area: String,
    direct: none,
    values: none,
}
