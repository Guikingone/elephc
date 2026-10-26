//! Purpose:
//! Registers the eval binding for compiler-owned PHP engine deprecations.
//!
//! Called from:
//! - The shared builtin registry's direct and evaluated-value dispatch.
//!
//! Key details:
//! - Shares error-handler state with public diagnostics and fixes the level to E_DEPRECATED.

eval_builtin! { contract: "__elephc_deprecated", area: Core, direct: Core, values: Core }
