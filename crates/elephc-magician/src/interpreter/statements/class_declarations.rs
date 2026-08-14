//! Purpose:
//! Registers eval class declarations and validates their direct parent.
//!
//! Called from:
//! - Statement dispatch for class and anonymous-class declarations.
//!
//! Key details:
//! - Registration occurs only after trait expansion and declaration validation succeed.

use super::*;

/// Registers an eval-declared class in the dynamic class table.
pub(in crate::interpreter) fn execute_class_decl_stmt(
    class: &EvalClass,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let name = class.name().trim_start_matches('\\');
    let context_has_class = context.has_class(name);
    let context_has_interface = context.has_interface(name);
    let context_has_trait = context.has_trait(name);
    let context_has_enum = context.has_enum(name);
    let runtime_has_class = trace_class_decl_result(
        class,
        "runtime_class_exists",
        values.class_exists(name),
    )?;
    let runtime_has_interface = trace_class_decl_result(
        class,
        "runtime_interface_exists",
        eval_runtime_interface_exists(name, values),
    )?;
    let runtime_has_trait =
        trace_class_decl_result(class, "runtime_trait_exists", values.trait_exists(name))?;
    let runtime_has_enum =
        trace_class_decl_result(class, "runtime_enum_exists", values.enum_exists(name))?;
    if !context_has_class
        && !context_has_interface
        && !context_has_trait
        && !context_has_enum
        && runtime_has_class
        && !runtime_has_interface
        && !runtime_has_trait
        && !runtime_has_enum
        && context.executing_include()
        && context.claim_aot_include_classlike(name)
    {
        trace_aot_include_class_decl(class);
        return Ok(());
    }
    if context_has_class
        || context_has_interface
        || context_has_trait
        || context_has_enum
        || runtime_has_class
        || runtime_has_interface
        || runtime_has_trait
        || runtime_has_enum
    {
        trace_class_decl_duplicate(
            class,
            [
                context_has_class,
                context_has_interface,
                context_has_trait,
                context_has_enum,
                runtime_has_class,
                runtime_has_interface,
                runtime_has_trait,
                runtime_has_enum,
            ],
        );
        return Err(EvalStatus::RuntimeFatal);
    }
    let class = trace_class_decl_result(
        class,
        "expand_traits",
        expand_eval_class_traits(class, context),
    )?
    .with_readonly_properties();
    let class = &class;
    trace_class_decl_result(
        class,
        "validate_modifiers",
        validate_eval_class_modifiers(class, context, values),
    )?;
    let native_parent = trace_class_decl_result(
        class,
        "validate_parent",
        validate_eval_class_parent(class, context, values),
    )?;
    for interface in class.interfaces() {
        if !context.has_interface(interface) && !eval_runtime_interface_exists(interface, values)? {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    trace_class_decl_result(
        class,
        "validate_throwable_interfaces",
        validate_eval_class_does_not_implement_throwable_interfaces(class, context),
    )?;
    trace_class_decl_result(
        class,
        "validate_enum_interfaces",
        validate_eval_class_does_not_implement_enum_interfaces(class, context),
    )?;
    trace_class_decl_result(
        class,
        "validate_interface_members",
        validate_declared_class_interface_members(class, context),
    )?;
    trace_class_decl_result(
        class,
        "validate_builtin_interface_members",
        validate_declared_class_builtin_interface_members(class, context),
    )?;
    trace_class_decl_result(
        class,
        "validate_aot_interface_members",
        validate_declared_class_aot_interface_members(class, context, values),
    )?;
    if !class.is_abstract() {
        trace_class_decl_result(
            class,
            "validate_concrete_requirements",
            validate_concrete_class_requirements(class, context),
        )?;
        trace_class_decl_result(
            class,
            "validate_builtin_interface_requirements",
            validate_concrete_class_builtin_interface_requirements(class, context),
        )?;
        trace_class_decl_result(
            class,
            "validate_aot_parent_requirements",
            validate_concrete_class_aot_parent_requirements(class, context, values),
        )?;
        trace_class_decl_result(
            class,
            "validate_aot_interface_requirements",
            validate_concrete_class_aot_interface_requirements(class, context, values),
        )?;
    }
    if context.define_class(class.clone()) {
        if let Some(parent) = native_parent.as_deref() {
            if !context.define_native_class_parent(class.name(), parent) {
                trace_class_decl_stage(class, "define_native_parent", EvalStatus::RuntimeFatal);
                return Err(EvalStatus::RuntimeFatal);
            }
        }
        trace_class_decl_result(
            class,
            "initialize_constants",
            initialize_eval_declared_constants(
                class.name(),
                class.constants(),
                context,
                scope,
                values,
            ),
        )?;
        trace_class_decl_result(
            class,
            "initialize_static_properties",
            initialize_eval_static_properties(class, context, scope, values),
        )
    } else {
        trace_class_decl_stage(class, "define_class", EvalStatus::RuntimeFatal);
        Err(EvalStatus::RuntimeFatal)
    }
}

/// Preserves a class-declaration result while tracing the stage that rejected it.
fn trace_class_decl_result<T>(
    class: &EvalClass,
    stage: &str,
    result: Result<T, EvalStatus>,
) -> Result<T, EvalStatus> {
    if let Err(status) = result.as_ref() {
        if std::env::var_os("ELEPHC_EVAL_TRACE").is_some() {
            eprintln!(
                "[elephc-eval-trace] phase=class_decl_error class={:?} stage={stage} status={status:?}",
                class.name(),
            );
        }
    }
    result
}

/// Emits one opt-in class-declaration stage failure without changing control flow.
fn trace_class_decl_stage(class: &EvalClass, stage: &str, status: EvalStatus) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    eprintln!(
        "[elephc-eval-trace] phase=class_decl_error class={:?} stage={stage} status={status:?}",
        class.name(),
    );
}

/// Traces which declaration registry already contains a class-like name.
fn trace_class_decl_duplicate(class: &EvalClass, matches: [bool; 8]) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    eprintln!(
        "[elephc-eval-trace] phase=class_decl_duplicate class={:?} context_class={} context_interface={} context_trait={} context_enum={} runtime_class={} runtime_interface={} runtime_trait={} runtime_enum={}",
        class.name(),
        matches[0],
        matches[1],
        matches[2],
        matches[3],
        matches[4],
        matches[5],
        matches[6],
        matches[7],
    );
}

/// Traces an include declaration already implemented by the compiled AOT class.
fn trace_aot_include_class_decl(class: &EvalClass) {
    if std::env::var_os("ELEPHC_EVAL_TRACE").is_none() {
        return;
    }
    eprintln!(
        "[elephc-eval-trace] phase=class_decl_aot_include class={:?}",
        class.name(),
    );
}

/// Validates an eval class parent and returns an AOT parent name when the parent is runtime-backed.
pub(super) fn validate_eval_class_parent(
    class: &EvalClass,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<String>, EvalStatus> {
    let Some(parent) = class.parent() else {
        return Ok(None);
    };
    let parent = context
        .resolve_class_name(parent)
        .unwrap_or_else(|| parent.trim_start_matches('\\').to_string());
    if let Some(parent_class) = context.class(&parent) {
        if parent_class.is_final()
            || parent_class.is_readonly_class() != class.is_readonly_class()
            || context.class_is_a(&parent, class.name(), false)
        {
            return Err(EvalStatus::RuntimeFatal);
        }
        return Ok(None);
    }
    let Some((parent_is_final, parent_is_readonly)) =
        eval_reflection_aot_class_inheritance_modifiers(&parent, values)?
    else {
        return Err(EvalStatus::RuntimeFatal);
    };
    if parent_is_final
        || parent_is_readonly != class.is_readonly_class()
        || native_class_is_a(&parent, class.name(), context)
    {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(Some(parent))
}

/// Registers one eval anonymous class expression if this execution has not seen it yet.
pub(in crate::interpreter) fn ensure_eval_anonymous_class_decl(
    class: &EvalClass,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if !class.is_anonymous() {
        return Err(EvalStatus::RuntimeFatal);
    }
    if let Some(existing) = context.class(class.name()) {
        return if existing.is_anonymous() {
            Ok(())
        } else {
            Err(EvalStatus::RuntimeFatal)
        };
    }
    execute_class_decl_stmt(class, context, scope, values)
}
