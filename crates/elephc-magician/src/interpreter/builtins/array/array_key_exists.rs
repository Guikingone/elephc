//! Purpose:
//! Joins Magician to the shared runtime builtin contract for `array_key_exists`.
//!
//! Called from:
//! - `crate::interpreter::builtins::registry` inventory assembly.
//!
//! Key details:
//! - Behavior dispatches by `RuntimeBuiltinId` through the versioned
//!   generated-runtime boxed-cell ABI; no Magician algorithm lives here.

eval_builtin! {
    contract: "array_key_exists",
    area: Array,
    direct: none,
    values: none,
}
