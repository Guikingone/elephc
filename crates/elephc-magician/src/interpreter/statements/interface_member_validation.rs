//! Purpose:
//! Validates declared methods and properties against interface member contracts.
//!
//! Called from:
//! - Class declaration validation before abstract-requirement collection.
//!
//! Key details:
//! - Eval, builtin, and AOT interface signatures use the same compatibility checks.

use super::*;

/// Validates declared or inherited class members that already cover eval interface contracts.
pub(super) fn validate_declared_class_interface_members(
    class: &EvalClass,
    context: &ElephcEvalContext,
) -> Result<(), EvalStatus> {
    for interface in pending_class_contract_interface_names(class, context) {
        if !context.has_interface(&interface) {
            continue;
        }
        validate_declared_class_interface_methods(class, &interface, context)?;
        validate_declared_class_interface_properties(class, &interface, context)?;
    }
    Ok(())
}

/// Validates declared class methods against PHP builtin runtime interface contracts.
pub(super) fn validate_declared_class_builtin_interface_members(
    class: &EvalClass,
    context: &ElephcEvalContext,
) -> Result<(), EvalStatus> {
    for (requirement_owner, requirement) in
        pending_class_builtin_interface_method_requirements(class, context)
    {
        let Some((declaring_class, method)) =
            pending_class_method(class, requirement.name(), context)
        else {
            continue;
        };
        if method.visibility() != EvalVisibility::Public
            || method.is_static() != requirement.is_static()
            || !class_method_satisfies_builtin_interface_signature(
                &method,
                &declaring_class,
                &requirement,
                &requirement_owner,
                Some(class),
                context,
            )
        {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    Ok(())
}

/// Validates declared class methods against generated/AOT interface contracts.
pub(super) fn validate_declared_class_aot_interface_members(
    class: &EvalClass,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    for requirement in pending_class_aot_interface_method_requirements(class, context, values)? {
        let Some((declaring_class, method)) = pending_class_method(class, &requirement.name, context)
        else {
            continue;
        };
        if !class_method_satisfies_aot_interface_requirement(
            &method,
            &declaring_class,
            &requirement,
            Some(class),
            context,
            false,
        ) {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    for (requirement_owner, requirement) in
        pending_class_aot_interface_property_requirements(class, context, values)?
    {
        let Some((declaring_class, property)) =
            pending_class_property_with_owner(class, requirement.name(), context)
        else {
            continue;
        };
        if !class_property_can_cover_interface_contract(
            &property,
            &declaring_class,
            &requirement,
            &requirement_owner,
            Some(class),
            context,
        ) {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    Ok(())
}

/// Validates class methods present for an eval interface, even on abstract classes.
pub(super) fn validate_declared_class_interface_methods(
    class: &EvalClass,
    interface_name: &str,
    context: &ElephcEvalContext,
) -> Result<(), EvalStatus> {
    for (requirement_owner, requirement) in
        context.interface_method_requirements_with_owners(interface_name)
    {
        let Some((declaring_class, method)) =
            pending_class_method(class, requirement.name(), context)
        else {
            continue;
        };
        if method.visibility() != EvalVisibility::Public
            || method.is_static() != requirement.is_static()
            || !class_method_satisfies_interface_signature(
                &method,
                &declaring_class,
                &requirement,
                &requirement_owner,
                Some(class),
                context,
            )
        {
            trace_declared_eval_interface_method_mismatch(
                class,
                interface_name,
                &requirement_owner,
                &requirement,
                &declaring_class,
                &method,
            );
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    Ok(())
}

/// Emits an opt-in trace when an eval class method violates an eval interface contract.
fn trace_declared_eval_interface_method_mismatch(
    class: &EvalClass,
    interface_name: &str,
    requirement_owner: &str,
    requirement: &EvalInterfaceMethod,
    declaring_class: &str,
    method: &EvalClassMethod,
) {
    if !crate::eval_trace::enabled() {
        return;
    }
    eprintln!(
        "[elephc-eval-trace] phase=interface_method_mismatch class={:?} interface={interface_name:?} owner={requirement_owner:?} method={:?} declaring_class={declaring_class:?} actual_visibility={:?} required_static={} actual_static={}",
        class.name(),
        requirement.name(),
        method.visibility(),
        requirement.is_static(),
        method.is_static(),
    );
}

/// Validates class properties present for an eval interface, even on abstract classes.
pub(super) fn validate_declared_class_interface_properties(
    class: &EvalClass,
    interface_name: &str,
    context: &ElephcEvalContext,
) -> Result<(), EvalStatus> {
    for (requirement_owner, requirement) in
        context.interface_property_requirements_with_owners(interface_name)
    {
        let Some((declaring_class, property)) =
            pending_class_property_with_owner(class, requirement.name(), context)
        else {
            continue;
        };
        if !class_property_can_cover_interface_contract(
            &property,
            &declaring_class,
            &requirement,
            &requirement_owner,
            Some(class),
            context,
        ) {
            return Err(EvalStatus::RuntimeFatal);
        }
    }
    Ok(())
}

/// Returns a method from a pending class or one of its already registered parents.
pub(super) fn pending_class_method(
    class: &EvalClass,
    method_name: &str,
    context: &ElephcEvalContext,
) -> Option<(String, EvalClassMethod)> {
    if let Some(method) = class.method(method_name) {
        return Some((class.name().to_string(), method.clone()));
    }
    class
        .parent()
        .and_then(|parent| context.class_method(parent, method_name))
}

/// Validates one method declaration against inherited eval method metadata.
pub(super) fn validate_method_parent_override(
    class: &EvalClass,
    method: &EvalClassMethod,
    context: &ElephcEvalContext,
) -> Result<(), EvalStatus> {
    let Some(parent) = class.parent() else {
        return Ok(());
    };
    let Some((parent_declaring_class, parent_method)) = context.class_method(parent, method.name())
    else {
        return Ok(());
    };
    let is_constructor = method.name().eq_ignore_ascii_case("__construct");
    if parent_method.visibility() == EvalVisibility::Private && !is_constructor {
        return Ok(());
    }
    if parent_method.is_static() != method.is_static() {
        return Err(EvalStatus::RuntimeFatal);
    }
    if parent_method.is_final() {
        return Err(EvalStatus::RuntimeFatal);
    }
    if method.is_abstract() && !parent_method.is_abstract() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let prototype = if is_constructor {
        // A concrete implementation retains the original abstract constructor
        // contract. Interface contracts are validated independently below.
        let mut prototype = None;
        for owner in std::iter::once(parent_declaring_class.clone())
            .chain(context.class_parent_names(&parent_declaring_class))
        {
            if let Some((owner, candidate)) = context.class_own_method(&owner, method.name()) {
                if candidate.is_abstract() {
                    prototype = Some((owner, candidate));
                }
            }
        }
        prototype
    } else {
        None
    };
    let (prototype_owner, prototype_method) = match prototype.as_ref() {
        Some((owner, method)) => (owner, method),
        None if is_constructor => return Ok(()),
        None => (&parent_declaring_class, &parent_method),
    };
    if method_visibility_rank(method.visibility())
        < method_visibility_rank(parent_method.visibility())
    {
        return Err(EvalStatus::RuntimeFatal);
    }
    if !class_method_signature_accepts(
        method,
        class.name(),
        prototype_method,
        prototype_owner,
        Some(class),
        context,
    ) {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(())
}

/// Validates one method declaration against inherited generated/AOT method metadata.
pub(super) fn validate_method_aot_parent_override(
    class: &EvalClass,
    method: &EvalClassMethod,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let Some(parent) = pending_class_native_parent_name(class, context) else {
        return Ok(());
    };
    if !values.class_exists(&parent)? {
        return Ok(());
    }
    let Some(flags) = values.reflection_method_flags(&parent, method.name())? else {
        return Ok(());
    };
    if flags & EVAL_REFLECTION_MEMBER_FLAG_PRIVATE != 0
        && !method.name().eq_ignore_ascii_case("__construct")
    {
        return Ok(());
    }
    let parent_is_static = flags & EVAL_REFLECTION_MEMBER_FLAG_STATIC != 0;
    if parent_is_static != method.is_static() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let parent_visibility = eval_aot_method_visibility(flags);
    if method_visibility_rank(method.visibility()) < method_visibility_rank(parent_visibility) {
        return Err(EvalStatus::RuntimeFatal);
    }
    if flags & EVAL_REFLECTION_MEMBER_FLAG_FINAL != 0 {
        return Err(EvalStatus::RuntimeFatal);
    }
    if method.is_abstract() && flags & EVAL_REFLECTION_MEMBER_FLAG_ABSTRACT == 0 {
        return Err(EvalStatus::RuntimeFatal);
    }
    let Some(required) = eval_aot_method_signature_requirement(
        &parent,
        method.name(),
        parent_is_static,
        context,
        values,
    )? else {
        return Ok(());
    };
    if !class_method_satisfies_interface_signature(
        method,
        class.name(),
        &required,
        &eval_aot_method_declaring_class(&parent, method.name(), values)?,
        Some(class),
        context,
    ) {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(())
}

/// Returns the eval visibility represented by generated/AOT reflection flags.
pub(super) fn eval_aot_method_visibility(flags: u64) -> EvalVisibility {
    if flags & EVAL_REFLECTION_MEMBER_FLAG_PRIVATE != 0 {
        EvalVisibility::Private
    } else if flags & EVAL_REFLECTION_MEMBER_FLAG_PROTECTED != 0 {
        EvalVisibility::Protected
    } else if flags & EVAL_REFLECTION_MEMBER_FLAG_PUBLIC != 0 {
        EvalVisibility::Public
    } else {
        EvalVisibility::Public
    }
}

/// Returns the generated/AOT declaring class for one reflected method.
pub(super) fn eval_aot_method_declaring_class(
    class_name: &str,
    method_name: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    values
        .reflection_method_declaring_class(class_name, method_name)
        .map(|declaring_class| declaring_class.unwrap_or_else(|| class_name.to_string()))
}

/// Returns a generated/AOT parent method signature as an eval method requirement.
pub(super) fn eval_aot_method_signature_requirement(
    class_name: &str,
    method_name: &str,
    is_static: bool,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<EvalInterfaceMethod>, EvalStatus> {
    let declaring_class = eval_aot_method_declaring_class(class_name, method_name, values)?;
    let signature = if is_static {
        context.native_static_method_signature(&declaring_class, method_name)
    } else {
        context.native_method_signature(&declaring_class, method_name)
    };
    Ok(signature.map(|signature| {
        eval_native_signature_interface_method(method_name, is_static, &signature)
    }))
}

/// Returns whether an override may stand in for a declaration that returns BY REFERENCE.
///
/// php's rule is one-directional: an override may ADD `&` where the declaration it replaces does
/// not ask for it, but may never DROP one it does. `php -n` 8.5.6 refuses that with
/// `Declaration of C::f() must be compatible with & P::f()` -- note the `&` in front of the
/// PARENT signature, which is how the message says which side wanted it.
///
/// It lives here, called from both the interface path and the parent-class path, because it was
/// written once for interfaces and the class path silently had no rule at all: a subclass could
/// replace a by-reference parent method with a by-value one and nothing said so.
pub(super) fn override_by_ref_return_is_compatible(
    override_returns_by_ref: bool,
    required_returns_by_ref: bool,
) -> bool {
    override_returns_by_ref || !required_returns_by_ref
}

/// Returns whether one eval class method can accept every call accepted by its parent method.
pub(super) fn class_method_signature_accepts(
    method: &EvalClassMethod,
    method_owner: &str,
    required: &EvalClassMethod,
    required_owner: &str,
    pending_class: Option<&EvalClass>,
    context: &ElephcEvalContext,
) -> bool {
    override_by_ref_return_is_compatible(method.returns_by_ref(), required.returns_by_ref())
        && method_signature_accepts(
        method.params().len(),
        method.parameter_defaults(),
        method.parameter_is_by_ref(),
        method.parameter_is_variadic(),
        required.params().len(),
        required.parameter_defaults(),
        required.parameter_is_by_ref(),
        required.parameter_is_variadic(),
    ) && method_parameter_type_signature_accepts(
        method.parameter_types(),
        method.parameter_is_variadic(),
        method_owner,
        required.parameter_types(),
        required.parameter_is_variadic(),
        required_owner,
        required.params().len(),
        pending_class,
        context,
    ) && method_return_type_signature_accepts(
        method.return_type(),
        method_owner,
        required.return_type(),
        required_owner,
        pending_class,
        context,
    )
}

/// Returns a comparable rank where larger means less restrictive visibility.
pub(super) fn method_visibility_rank(visibility: EvalVisibility) -> u8 {
    match visibility {
        EvalVisibility::Private => 1,
        EvalVisibility::Protected => 2,
        EvalVisibility::Public => 3,
    }
}
