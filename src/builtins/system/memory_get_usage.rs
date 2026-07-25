//! Purpose:
//! Home of the PHP `memory_get_usage` builtin and its runtime heap-accounting contract.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The default form reports live Elephc heap payload bytes.
//! - `real_usage=true` reports committed arena bytes, including reusable freed blocks.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "memory_get_usage",
    area: System,
    params: [real_usage: Bool = DefaultSpec::Bool(false)],
    returns: Int,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::MemoryGetUsage,
    ),
    summary: "Returns current Elephc heap memory usage in bytes.",
    examples: &["echo memory_get_usage();", "echo memory_get_usage(true);"],
    php_manual: "function.memory-get-usage",
}
