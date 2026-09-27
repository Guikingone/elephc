//! Purpose:
//! Declarative eval registry entry for PHP's `mb_strtolower()`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string` and the declarative direct/values hooks, which
//!   route the pair through the `Iconv` hook group because the engine lives in `elephc-iconv`.
//!
//! Key details:
//! - Evaluation shares `mb_strtoupper`'s `eval_mb_convert_case_result` with
//!   `CaseMode::Lower`, so the final-sigma rule and the `ValueError` text are the AOT ones.

eval_builtin! {
    contract: "mb_strtolower",
    area: String,
    direct: Iconv,
    values: Iconv,
}
