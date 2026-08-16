//! Purpose:
//! Home of the PHP `sscanf` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The scanner itself is `__elephc_scanf`, declared by `crate::scanf_prelude` in elephc-PHP;
//!   this builtin only validates the call and lowers to a direct call against it. The engine
//!   used to be `__rt_sscanf`, ~700 lines of per-architecture assembly that pushed every match
//!   back as a STRING slice — `sscanf('77 xx', '%d %d')` answered `['77', '']` where php answers
//!   `[77, NULL]` — and knew none of `%i`/`%u`/`%x`/`%o`/`%c`/`%n`/`%[...]`, widths, `*`
//!   suppression, or the EOF result. See the prelude for the measured php rules.
//! - `check` returns `array|null` (`Array<Mixed>` plus `Void`) because that is php's 2-argument
//!   result: an entry per non-suppressed conversion — int, float, string or `null` — and a bare
//!   `null` when the scan hits end of input before assigning anything. `returns: Mixed` stays in
//!   the contract because the macro's scalar `returns:` field cannot express it.
//! - The by-ref `$vars` output form is REFUSED rather than mis-executed. php assigns each field
//!   through the reference and returns the field COUNT; this backend has no way to express that
//!   yet — `ParamSpec` carries `by_ref`, but `variadic: Some("vars")` is a bare NAME with no
//!   by-ref marker, so the checker never introduces the variables and the lowering never writes
//!   them. Left accepted, the call compiled and silently did the wrong thing on both counts at
//!   once: measured with `php -n` (8.5.6),
//!   `$name = "unset"; $age = -1; $n = sscanf("alice 30", "%s %d", $name, $age);` gives php
//!   `int(2)` / `"alice"` / `int(30)`, while this backend returned the ARRAY
//!   `["alice", "30"]` and left BOTH variables untouched.

use crate::builtins::semantics::{
    BuiltinCallablePolicy, BuiltinEffects, BuiltinLowering, BuiltinLoweringContext,
    BuiltinLoweringError, BuiltinRequirements, BuiltinResultOwnership, BuiltinResultType,
    BuiltinRuntimeFunctions, BuiltinSemantics, BuiltinTargetStrategy, BuiltinTargetSupport,
    BuiltinValidation, LoweredBuiltinValue, NormalizedBuiltinCall,
};
use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::ir::{Immediate, Op};
use crate::types::PhpType;

/// The elephc-PHP prelude function both scanf builtins scan through.
pub(crate) const SCANF_ENGINE_FUNCTION: &str = "__elephc_scanf";

builtin! {
    contract: "sscanf",
    check: check,
    semantics: BuiltinSemantics {
        validation: BuiltinValidation::CheckerHook { check, lazy: false },
        result_type: BuiltinResultType::Checked,
        effects: BuiltinEffects::Shared(engine_call_effects),
        result_ownership: BuiltinResultOwnership::Fresh,
        requirements: BuiltinRequirements::Static(&[]),
        target_strategy: BuiltinTargetStrategy::EirPrimitive,
        target_support: BuiltinTargetSupport::All,
        runtime_functions: BuiltinRuntimeFunctions::None,
        argument_lowering: crate::builtins::semantics::BuiltinArgumentLowering::Standard,
        callable: BuiltinCallablePolicy::StaticOnly(
            "sscanf is scanned by an injected prelude function, which a runtime-selected callable cannot reach",
        ),
        lowering: BuiltinLowering::Eir(lower),
    },
}

/// Returns php's 2-argument `sscanf` result type: `array|null`.
///
/// A check hook is required because the `builtin!` macro cannot express a parameterized
/// array-or-null return type inline. The hook also refuses the by-ref `$vars` output form,
/// which this backend cannot express and previously mis-executed in silence.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    reject_by_ref_vars(cx.name, cx.args.len(), cx.span)?;
    Ok(scanf_result_type())
}

/// Returns the effect contract of a scanf builtin: those of the prelude call it lowers to.
///
/// Shared with `fscanf()`, whose prelude entry point also reads from the stream through
/// `fgets()` — a call effect already covers both.
pub(crate) fn engine_call_effects(
    _input: &crate::builtins::semantics::BuiltinSemanticInput<'_>,
) -> crate::ir::Effects {
    Op::Call.default_effects()
}

/// Returns `array|null`, php's result for a scan that may hit end of input.
///
/// Shared with `fscanf()`, which adds `false` for a stream already at EOF.
pub(crate) fn scanf_result_type() -> PhpType {
    PhpType::Union(vec![
        PhpType::Array(Box::new(PhpType::Mixed)),
        PhpType::Void,
    ])
}

/// Lowers `sscanf(string, format)` to a direct call into the injected scanf prelude.
fn lower(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
) -> Result<LoweredBuiltinValue, BuiltinLoweringError> {
    lower_scanf_engine_call(ctx, call, SCANF_ENGINE_FUNCTION)
}

/// Emits `Op::Call` against one prelude scanf entry point with the call's two operands.
///
/// Shared by `sscanf()` and `fscanf()`, whose only difference is which prelude function
/// receives the pair — the subject string, or the stream the line is read from.
pub(crate) fn lower_scanf_engine_call(
    ctx: &mut dyn BuiltinLoweringContext,
    call: &NormalizedBuiltinCall<'_>,
    function: &str,
) -> Result<LoweredBuiltinValue, BuiltinLoweringError> {
    let operands = vec![call.operand(0)?, call.operand(1)?];
    let name = ctx.intern_function_name(function);
    Ok(ctx.emit_value(
        Op::Call,
        operands,
        Some(Immediate::Data(name)),
        call.result_type.clone(),
        Op::Call.default_effects(),
        Some(call.span),
    ))
}

/// Refuses the `scanf` by-ref `$vars` output form for `sscanf()`/`fscanf()`.
///
/// Shared by both builtins so the two stay on one wording. `arg_count` is the full argument
/// count including the two leading required parameters, so anything past 2 is a `$vars` entry.
pub(crate) fn reject_by_ref_vars(
    name: &str,
    arg_count: usize,
    span: crate::span::Span,
) -> Result<(), CompileError> {
    if arg_count <= 2 {
        return Ok(());
    }
    Err(CompileError::new(
        span,
        &format!(
            "{}(): the by-ref $vars output form is not supported; \
             call it with only the format and read the returned array",
            name
        ),
    ))
}
