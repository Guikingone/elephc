//! Purpose:
//! Declarative eval registry entry for `strncasecmp`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the length-limited compare in
//!   `super::strncmp`, which both names share.

eval_builtin! {
    contract: "strncasecmp",
    area: String,
    direct: StringCompareNCase,
    values: StringCompareNCase,
}
