//! Purpose:
//! Joins `memory_get_peak_usage()` to the shared boxed runtime dispatcher.
//!
//! Called from:
//! - Magician builtin registry inventory assembly.
//!
//! Key details:
//! - Reads the same native heap as compiled PHP through `RuntimeBuiltinId`.

eval_builtin! {
    contract: "memory_get_peak_usage",
    area: Core,
    direct: none,
    values: none,
}
