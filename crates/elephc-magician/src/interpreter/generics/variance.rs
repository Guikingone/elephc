//! Purpose:
//! Validates generic variance promises and relates compatible object specializations.
//!
//! Called from:
//! - Template registration and structural argument/return validation.
//!
//! Key details:
//! - Constructors and private members are outside the widened public surface.
//! - Widening preserves AOT's storage rule: only object type arguments can differ.

use super::*;

/// Returns whether a type argument is an object reference in AOT storage.
pub(super) fn object_type(ty: &Type) -> bool {
    match ty {
        Type::Apply(name, _) => name != "array" && name != "buffer" && name != "ptr",
        Type::Atom(name) => !["int", "float", "string", "bool", "false", "null", "void", "never", "mixed", "array", "callable", "iterable", "object", "ptr"].contains(&name.as_str()),
        _ => false,
    }
}

/// Separates constructor identity slots from a generic class's public type arguments.
pub(super) fn generic_class_arguments<'a>(name: &str, args: &'a [Type], context: &ElephcEvalContext) -> &'a [Type] {
    if let Some(template) = context.generic_template(name) {
        let count = template.parameters.len();
        if count != 0 && !template.constructor_parameters.is_empty()
            && args.len() == count + template.constructor_parameters.len() { return &args[..count]; }
    }
    args
}

/// Removes constructor identity slots recursively before matching a declared generic shape.
pub(super) fn generic_inference_type(ty: &Type, context: &ElephcEvalContext) -> Type {
    match ty {
        Type::Apply(name, args) => Type::Apply(name.clone(), generic_class_arguments(name, args, context).iter().map(|ty| generic_inference_type(ty, context)).collect()),
        Type::Nullable(inner) => Type::Nullable(Box::new(generic_inference_type(inner, context))),
        Type::Union(types) => Type::Union(types.iter().map(|ty| generic_inference_type(ty, context)).collect()),
        Type::Intersection(types) => Type::Intersection(types.iter().map(|ty| generic_inference_type(ty, context)).collect()),
        Type::Callable(params, ret) => Type::Callable(params.iter().map(|ty| generic_inference_type(ty, context)).collect(), Box::new(generic_inference_type(ret, context))),
        _ => ty.clone(),
    }
}

/// Relates retained signature types using dynamic hierarchy metadata.
pub(in crate::interpreter) fn generic_signature_accepts(actual: &Type, expected: &Type, context: &ElephcEvalContext) -> bool {
    if actual == expected || object_type(actual) && object_type(expected) && actual.to_string().eq_ignore_ascii_case(&expected.to_string()) { return true; }
    if let (Type::Atom(actual), Type::Atom(expected)) = (actual, expected) {
        return actual.eq_ignore_ascii_case(expected) || context.class_is_a(actual, expected, false);
    }
    let (Type::Apply(actual_name, actual_args), Type::Apply(expected_name, expected_args)) = (actual, expected) else {
        return context.class_is_a(&actual.to_string(), &expected.to_string(), false);
    };
    let Some(template) = context.generic_template(actual_name) else { return false; };
    let actual_args = generic_class_arguments(actual_name, actual_args, context);
    let expected_args = generic_class_arguments(expected_name, expected_args, context);
    actual_name.eq_ignore_ascii_case(expected_name) && actual_args.len() == expected_args.len()
        && template.parameters.len() == actual_args.len()
        && template.parameters.iter().zip(actual_args.iter().zip(expected_args)).all(|(param, (actual, expected))| {
            if actual == expected { return true; }
            if !object_type(actual) || !object_type(expected) { return false; }
            match param.variance {
                1 => generic_signature_accepts(actual, expected, context),
                -1 => generic_signature_accepts(expected, actual, context),
                _ => false,
            }
        })
}

/// Checks whether any occurrence of a slot violates its declared polarity.
fn violates(ty: &Type, slot: &str, variance: i8, polarity: i8, context: &ElephcEvalContext) -> bool {
    match ty {
        Type::Atom(name) => name == slot && variance != polarity,
        Type::Nullable(inner) => violates(inner, slot, variance, polarity, context),
        Type::Union(types) | Type::Intersection(types) => types.iter().any(|ty| violates(ty, slot, variance, polarity, context)),
        Type::Callable(params, ret) => params.iter().any(|ty| violates(ty, slot, variance, -polarity, context))
            || violates(ret, slot, variance, polarity, context),
        Type::Apply(name, args) => {
            let template = context.generic_template(name);
            args.iter().enumerate().any(|(index, ty)| {
                let nested = if name == "array" || name == "buffer" { polarity }
                    else { polarity * template.and_then(|template| template.parameters.get(index)).map_or(0, |param| param.variance) };
                violates(ty, slot, variance, nested, context)
            })
        }
    }
}

/// Checks one member type after substituting inherited template arguments.
fn check_member(ty: &EvalParameterType, slot: &str, variance: i8, polarity: i8, bindings: &Bindings, context: &ElephcEvalContext) -> bool {
    !violates(&generic_type(ty).substitute(bindings), slot, variance, polarity, context)
}

/// Validates own and inherited public member types for one marked slot.
fn verify_slot(declaration: &EvalStmt, slot: &str, variance: i8, bindings: &Bindings, context: &ElephcEvalContext, depth: usize) -> bool {
    if depth > 16 { return true; }
    let mut inherited = Vec::new();
    match declaration {
        EvalStmt::ClassDecl(class) => {
            for property in class.properties().iter().filter(|property| property.visibility() != EvalVisibility::Private) {
                if property.property_type().is_some_and(|ty| !check_member(ty, slot, variance, if property.is_readonly() { 1 } else { 0 }, bindings, context)) { return false; }
            }
            for method in class.methods().iter().filter(|method| method.visibility() != EvalVisibility::Private && !method.name().eq_ignore_ascii_case("__construct")) {
                if method.generic_template().is_some_and(|template| template.parameters.iter().any(|param| param.name == slot)) { continue; }
                if method.parameter_types().iter().flatten().any(|ty| !check_member(ty, slot, variance, -1, bindings, context))
                    || method.return_type().is_some_and(|ty| !check_member(ty, slot, variance, 1, bindings, context)) { return false; }
            }
            inherited.extend(class.parent().into_iter().chain(class.interfaces().iter().map(String::as_str)).chain(class.traits().iter().map(String::as_str)));
        }
        EvalStmt::InterfaceDecl(interface) => {
            for property in interface.properties() {
                if property.property_type().is_some_and(|ty| !check_member(ty, slot, variance, if property.requires_set() { 0 } else { 1 }, bindings, context)) { return false; }
            }
            for method in interface.methods() {
                if method.parameter_types().iter().flatten().any(|ty| !check_member(ty, slot, variance, -1, bindings, context))
                    || method.return_type().is_some_and(|ty| !check_member(ty, slot, variance, 1, bindings, context)) { return false; }
            }
            inherited.extend(interface.parents().iter().map(String::as_str));
        }
        _ => return true,
    }
    for relation in inherited {
        let Ok(Type::Apply(name, arguments)) = crate::parser::generics::parse_type_string(relation) else { continue; };
        let Some(parent) = context.generic_template(&name) else { continue; };
        let parent_bindings = parent.parameters.iter().map(|param| param.name.clone()).zip(arguments.iter().map(|ty| ty.substitute(bindings))).collect();
        if !verify_slot(&parent.prototype, slot, variance, &parent_bindings, context, depth + 1) { return false; }
    }
    true
}

/// Rejects invalid variance promises before any template body can execute.
pub(in crate::interpreter) fn verify_generic_variance(template: &EvalGenericTemplate, context: &mut ElephcEvalContext, values: &mut impl RuntimeValueOps) -> Result<(), EvalStatus> {
    for param in &template.parameters {
        if param.variance != 0 && !verify_slot(&template.prototype, &param.name, param.variance, &Vec::new(), context, 0) {
            return eval_throw_type_error(&format!("Generic '{}' uses type parameter <{}> in a position forbidden by its variance", template.name, param.name), context, values);
        }
    }
    Ok(())
}
