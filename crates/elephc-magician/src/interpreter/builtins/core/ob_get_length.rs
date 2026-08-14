//! Purpose:
//! Joins Magician to the shared runtime builtin contract for `ob_get_length`.
//!
//! Called from:
//! - `crate::interpreter::builtins::registry` inventory assembly.
//!
//! Key details:
//! - Behavior dispatches by `RuntimeBuiltinId` through the versioned
//!   generated-runtime boxed-cell ABI; no Magician algorithm lives here.

eval_builtin! {
    contract: "ob_get_length",
    area: Core,
    direct: none,
    values: none,
}
