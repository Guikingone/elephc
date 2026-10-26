//! Purpose:
//! Binds compiler-owned PHP engine deprecations to the shared builtin contract.
//!
//! Called from:
//! - Synthetic SPL storage constructors and exchangeArray bodies.
//!
//! Key details:
//! - The typed EIR operation always emits E_DEPRECATED; public trigger_error stays validated.

use super::core_support::core_builtin_home;

core_builtin_home!("__elephc_deprecated", InternalDeprecated);
