//! Purpose:
//! Validates schema validation declarations for the checker.
//! Turns parsed declarations into canonical metadata and rejects invalid contracts before code generation.
//!
//! Called from:
//! - `crate::types::checker::schema`
//!
//! Key details:
//! - Declaration metadata must align with name resolution, inheritance flattening, and runtime/codegen expectations.

use std::collections::{HashMap, HashSet};

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{Attribute, ClassMethod, Expr, ExprKind, StmtKind, Visibility};
use crate::types::traits::FlattenedClass;
use crate::types::{FunctionSig, PhpType};

use super::super::Checker;

/// Builds a `FunctionSig` from a parsed class method, resolving parameter and return type
/// annotations through the checker. Parameters without type hints default to `PhpType::Int`.
/// Validates that each declared parameter's default value is compatible with its resolved type.
/// Infers return type from method body when no return annotation is present.
pub(crate) fn build_method_sig(
    checker: &Checker,
    method: &ClassMethod,
) -> Result<FunctionSig, CompileError> {
    let params: Vec<(String, PhpType)> = method
        .params
        .iter()
        .map(|(n, type_ann, _, _)| {
            let ty = match type_ann {
                Some(type_ann) => checker.resolve_declared_param_type_hint(
                    type_ann,
                    method.span,
                    &format!("Method parameter ${}", n),
                )?,
                None => PhpType::Int,
            };
            Ok((n.clone(), ty))
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let defaults: Vec<Option<Expr>> = method.params.iter().map(|(_, _, d, _)| d.clone()).collect();
    let ref_params: Vec<bool> = method.params.iter().map(|(_, _, _, r)| *r).collect();
    for ((param_name, type_ann, default, _), (_, resolved_ty)) in
        method.params.iter().zip(params.iter())
    {
        if type_ann.is_some() {
            checker.validate_declared_default_type(
                resolved_ty,
                default.as_ref(),
                method.span,
                &format!("Method parameter ${}", param_name),
            )?;
        }
    }
    let return_type = match method.return_type.as_ref() {
        Some(type_ann) => checker.resolve_declared_return_type_hint(
            type_ann,
            method.span,
            &format!("Method '{}'", method.name),
        )?,
        None => super::super::infer_return_type_syntactic(&method.body),
    };
    let mut sig = Checker::callable_wrapper_sig(&FunctionSig {
        params,
        defaults,
        return_type,
        declared_return: method.return_type.is_some(),
        by_ref_return: method.by_ref_return,
        ref_params,
        declared_params: method
            .params
            .iter()
            .map(|(_, type_ann, _, _)| type_ann.is_some())
            .chain(method.variadic.iter().map(|_| method.variadic_type.is_some()))
            .collect(),
        variadic: method.variadic.clone(),
        deprecation: extract_deprecation(&method.attributes),
    });
    // A declared element type on the variadic (`int ...$xs`) constrains every collected argument.
    // `callable_wrapper_sig` defaults the variadic container to `array<mixed>`; refine it to the
    // declared element type so call validation enforces it.
    if let Some(variadic_type) = &method.variadic_type {
        let elem_ty = checker.resolve_declared_param_type_hint(
            variadic_type,
            method.span,
            &format!(
                "Method variadic parameter ${}",
                method.variadic.as_deref().unwrap_or_default()
            ),
        )?;
        if let Some((_, ty)) = sig.params.last_mut() {
            *ty = PhpType::Array(Box::new(elem_ty));
        }
    }
    Ok(sig)
}

/// Returns `Some(reason)` when the attribute list contains a `#[\Deprecated]`
/// marker, with `reason` set to the attribute's first string argument (or an
/// empty string if absent). Match is case-insensitive on the last segment of
/// the attribute name.
pub(crate) fn extract_deprecation(
    groups: &[crate::parser::ast::AttributeGroup],
) -> Option<String> {
    for group in groups {
        for attr in &group.attributes {
            if !matches_global_builtin_attribute(attr, "Deprecated") {
                continue;
            }
            let reason = attr.args.iter().find_map(|expr| match &expr.kind {
                ExprKind::StringLiteral(s) => Some(s.clone()),
                _ => None,
            });
            return Some(reason.unwrap_or_default());
        }
    }
    None
}

/// Returns `true` if `attr` is a global builtin attribute matching `builtin` by name.
/// Fully-qualified names must match exactly (case-insensitive); unqualified names
/// match the last segment case-insensitively. Used to detect `#[\Deprecated]` and similar.
pub(crate) fn matches_global_builtin_attribute(attr: &Attribute, builtin: &str) -> bool {
    let name = attr.name.as_canonical();
    if attr.name.is_fully_qualified() {
        return name.eq_ignore_ascii_case(builtin);
    }
    attr.name.is_unqualified() && name.eq_ignore_ascii_case(builtin)
}

/// Builds a mapping from constructor parameter index to property name for each parameter.
/// For each parameter, searches constructor body for `PropertyAssign` statements where
/// the right-hand side is a Variable with the same name as the parameter; if found,
/// returns `Some(property_name)`, otherwise `None`. Returns empty vec if no constructor.
pub(crate) fn build_constructor_param_map(methods: &[ClassMethod]) -> Vec<Option<String>> {
    let mut param_to_prop = Vec::new();
    if let Some(constructor) = methods
        .iter()
        .find(|m| php_symbol_key(&m.name) == "__construct")
    {
        param_to_prop = constructor
            .params
            .iter()
            .map(|(pname, _, _, _)| {
                for stmt in &constructor.body {
                    if let StmtKind::PropertyAssign {
                        property, value, ..
                    } = &stmt.kind
                    {
                        if let ExprKind::Variable(vn) = &value.kind {
                            if vn == pname {
                                return Some(property.clone());
                            }
                        }
                    }
                }
                None
            })
            .collect();
    }
    param_to_prop
}

/// Returns a numeric rank for visibility levels: `private=0`, `protected=1`, `public=2`.
/// Used to enforce that overriding methods are not less visible than the parent method.
pub(crate) fn visibility_rank(visibility: &Visibility) -> u8 {
    match visibility {
        Visibility::Private => 0,
        Visibility::Protected => 1,
        Visibility::Public => 2,
    }
}

/// Counts how many parameters in `sig` are required (have no default).
/// The variadic parameter, if present, is never considered required even if it has no default.
pub(crate) fn required_param_count(sig: &FunctionSig) -> usize {
    sig.defaults
        .iter()
        .enumerate()
        .filter(|(idx, default)| {
            if sig.variadic.is_some() && *idx + 1 == sig.defaults.len() {
                return false;
            }
            default.is_none()
        })
        .count()
}

/// Validates that `child_sig` is compatible with `parent_sig` for override purposes.
/// Checks parameter count, ref params, defaults layout, variadic flag, and required param count.
/// Reports errors with `context` and `kind` (e.g., "overriding method") in the message.
pub(crate) fn validate_signature_compatibility(
    span: crate::span::Span,
    owner_name: &str,
    method_name: &str,
    child_sig: &FunctionSig,
    parent_sig: &FunctionSig,
    kind: &str,
    context: &str,
) -> Result<(), CompileError> {
    if child_sig.params.len() != parent_sig.params.len() {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change parameter count when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    if child_sig.ref_params != parent_sig.ref_params {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change pass-by-reference parameters when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    let child_defaults: Vec<bool> = child_sig
        .defaults
        .iter()
        .map(|default| default.is_some())
        .collect();
    let parent_defaults: Vec<bool> = parent_sig
        .defaults
        .iter()
        .map(|default| default.is_some())
        .collect();
    if child_defaults != parent_defaults {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change optional parameter layout when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    if child_sig.variadic != parent_sig.variadic {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change variadic parameter shape when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    if required_param_count(child_sig) != required_param_count(parent_sig) {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change required parameter count when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    Ok(())
}

/// Returns `true` if `actual` is a compatible declared return type for `expected`.
/// Allows `PhpType::Never` (unreachable) as a wildcard match. Otherwise delegates to
/// `checker.type_accepts(expected, actual)` for standard subtype checking.
pub(crate) fn declared_return_type_compatible(
    checker: &Checker,
    expected: &PhpType,
    actual: &PhpType,
) -> bool {
    matches!(actual, PhpType::Never) || checker.type_accepts(expected, actual)
}

/// Returns `true` when `child` is the same type as `ancestor`, or a subtype of it —
/// i.e. `child` transitively `extends`/`implements` `ancestor`.
///
/// The subtype relationship is resolved **order-independently** from the complete
/// `class_map` (every class's `extends`/`implements` is known before any class body is
/// built) together with the already-fully-built interface table on `checker` (all
/// interfaces are built before any class). This matters because class-override
/// validation runs mid-schema-build, when a return type's class may not yet be
/// registered in `checker.classes`; walking only `checker.classes` (as `is_subclass_of`
/// does) would falsely reject legal covariant returns. Interface-to-interface edges
/// defer to `checker.interface_extends_interface`. A name absent from both `class_map`
/// and the interface table (e.g. a builtin class not registered in `class_map`) falls
/// back to the partially-built `checker` tables. Cycles are guarded with a visited set,
/// and an unknown/unrelated `child` is treated conservatively (returns `false`).
fn class_map_is_subtype(
    child: &str,
    ancestor: &str,
    class_map: &HashMap<String, FlattenedClass>,
    checker: &Checker,
) -> bool {
    if child == ancestor {
        return true;
    }
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = vec![child.to_string()];
    while let Some(current) = queue.pop() {
        if current == ancestor {
            return true;
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        // Interface chains are fully resolved before any class is built, so honor an
        // interface-extends-interface edge (also true when `current` == `ancestor`).
        if checker.interfaces.contains_key(&current)
            && checker.interface_extends_interface(&current, ancestor)
        {
            return true;
        }
        if let Some(flat) = class_map.get(&current) {
            if let Some(parent) = &flat.extends {
                queue.push(parent.clone());
            }
            for interface_name in &flat.implements {
                if interface_name == ancestor
                    || checker.interface_extends_interface(interface_name, ancestor)
                {
                    return true;
                }
                queue.push(interface_name.clone());
            }
        } else if checker.is_subclass_of(&current, ancestor)
            || checker.object_type_implements_interface(&current, ancestor)
        {
            // `current` is not a user-declared class in `class_map` (e.g. a builtin class
            // or interface already registered): consult the built `checker` tables.
            return true;
        }
    }
    false
}

/// Returns `true` when a child override's declared return type `actual` is a legal
/// **covariant** refinement of the parent's declared return type `expected` (PHP 7.4+).
///
/// Covariance means `actual` is the same as, or a subtype of, `expected`; a return type
/// may be narrowed but never widened. This accepts everything
/// `declared_return_type_compatible` already allows (exact match, `type_accepts`,
/// `Never`) and additionally accepts an object/interface `actual` that is a subtype of
/// `expected` per `class_map_is_subtype` (resolved order-independently so a not-yet-built
/// return-type class is not falsely rejected), plus nullable/union returns where every
/// `actual` member is covariant into some `expected` member. An `actual` that is a
/// supertype of, or unrelated to, `expected` is NOT accepted, so genuine
/// contravariant/incompatible returns keep erroring (return types are covariant, not
/// contravariant).
pub(crate) fn covariant_return_compatible(
    checker: &Checker,
    class_map: &HashMap<String, FlattenedClass>,
    expected: &PhpType,
    actual: &PhpType,
) -> bool {
    if declared_return_type_compatible(checker, expected, actual) {
        return true;
    }
    match (expected, actual) {
        (PhpType::Object(expected_name), PhpType::Object(actual_name)) => {
            class_map_is_subtype(actual_name, expected_name, class_map, checker)
        }
        // Nullable/union return: every child member must be covariant into the parent
        // union, which also enforces child nullability ⊆ parent nullability (a child
        // `null` member with no matching parent member is rejected below).
        (PhpType::Union(_), PhpType::Union(actual_members)) => actual_members
            .iter()
            .all(|member| covariant_return_compatible(checker, class_map, expected, member)),
        (PhpType::Union(expected_members), _) => expected_members
            .iter()
            .any(|member| covariant_return_compatible(checker, class_map, member, actual)),
        _ => false,
    }
}

/// Validates that `method` can override `parent_sig` in class `class_name`.
/// Builds the child signature via `build_method_sig`, skips validation for `__construct`,
/// checks signature compatibility, and ensures the child does not remove a declared
/// return type when the parent has one or make it incompatible.
pub(crate) fn validate_override_signature(
    checker: &Checker,
    class_map: &HashMap<String, FlattenedClass>,
    class_name: &str,
    method: &ClassMethod,
    parent_sig: &FunctionSig,
    is_static: bool,
) -> Result<(), CompileError> {
    let kind = if is_static { "static method" } else { "method" };
    let child_sig = build_method_sig(checker, method)?;
    if php_symbol_key(&method.name) == "__construct" {
        return Ok(());
    }
    validate_signature_compatibility(
        method.span,
        class_name,
        &method.name,
        &child_sig,
        parent_sig,
        kind,
        "overriding",
    )?;
    if parent_sig.declared_return && !child_sig.declared_return {
        return Err(CompileError::new(
            method.span,
            &format!(
                "Cannot override {} {}::{} without declaring a compatible return type (parent returns {})",
                kind, class_name, method.name, parent_sig.return_type
            ),
        ));
    }
    if parent_sig.declared_return
        && !covariant_return_compatible(
            checker,
            class_map,
            &parent_sig.return_type,
            &child_sig.return_type,
        )
    {
        return Err(CompileError::new(
            method.span,
            &format!(
                "Cannot override {} {}::{} with incompatible return type {} (parent returns {})",
                kind, class_name, method.name, child_sig.return_type, parent_sig.return_type
            ),
        ));
    }
    Ok(())
}
