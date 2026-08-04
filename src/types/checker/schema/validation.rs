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
use crate::parser::ast::{Attribute, ClassMethod, Expr, ExprKind, StmtKind, TypeExpr, Visibility};
use crate::types::traits::FlattenedClass;
use crate::types::{FunctionSig, PhpType};

use super::super::Checker;

/// Builds a `FunctionSig` from a parsed class method, resolving parameter and return type
/// annotations through the checker. Parameters without type hints default to `PhpType::Int`.
/// Validates that each declared parameter's default value is compatible with its resolved type.
/// Infers return type from method body when no return annotation is present. A generator method
/// (body contains `yield`) is seeded `Object("Generator")` regardless of the declared
/// `iterable`/`Generator`/`Traversable` hint so recursive generator-method calls resolve the
/// correct type before the method pass runs; the declared hint is validated by the method pass.
pub(crate) fn build_method_sig(
    checker: &Checker,
    method: &ClassMethod,
    declaring_type: &str,
) -> Result<FunctionSig, CompileError> {
    let method_key = php_symbol_key(&method.name);
    let params: Vec<(String, PhpType)> = method
        .params
        .iter()
        .enumerate()
        .map(|(i, (n, type_ann, _, _))| {
            // PHP's __unserialize($data) always receives the associative array
            // produced by __serialize(). A bare `array` hint resolves to an indexed
            // Array(Mixed) (rejecting $data['key']); type the first parameter as a
            // string/int-keyed assoc array so both the ABI and the body agree.
            // Scoped to user-defined methods (real span): synthetic SPL __unserialize
            // bodies are written for an indexed `array` and are called directly with
            // one (e.g. SplDoublyLinkedList), so they must keep Array(Mixed).
            if method_key == "__unserialize" && i == 0 && method.span.line != 0 {
                return Ok((
                    n.clone(),
                    PhpType::AssocArray {
                        key: Box::new(PhpType::Mixed),
                        value: Box::new(PhpType::Mixed),
                    },
                ));
            }
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
    let mut ref_params: Vec<bool> = method.params.iter().map(|(_, _, _, r)| *r).collect();
    for ((param_name, type_ann, default, _), (_, resolved_ty)) in
        method.params.iter().zip(params.iter())
    {
        if type_ann.is_some() {
            checker.validate_schema_parameter_default_type(
                resolved_ty,
                default.as_ref(),
                method.span,
                &format!("Method parameter ${}", param_name),
            )?;
        }
    }
    let return_type = match method.return_type.as_ref() {
        // No widening of a declared `callable` here: this pre-pass has no body to inspect, so it
        // cannot tell a descriptor-only body from one returning a string or an `[$obj, 'm']` pair.
        // `Checker::check_method_bodies` re-derives the effective return type from the collected
        // returns and applies `callable_return_slot_type` there.
        Some(type_ann) => checker.resolve_method_return_type_hint(
            type_ann,
            declaring_type,
            method.span,
            &format!("Method '{}'", method.name),
        )?,
        // A bodyless method (an interface method or an `abstract` declaration) has no body
        // to infer from and no declared return type. PHP treats such a method as untyped:
        // the implementor may return any value, so callers must see Mixed. Seeding Void here
        // (as `infer_return_type_syntactic` does for an empty body) would wrongly reject using
        // the overriding implementation's result as a value (e.g. `$c->get($id)::class`).
        None if !method.has_body => PhpType::Mixed,
        None => super::super::infer_return_type_syntactic(&method.body),
    };
    if method.variadic.is_some() {
        ref_params.push(method.variadic_by_ref);
    }
    let mut sig = Checker::callable_wrapper_sig(&FunctionSig {
        params,
        param_type_exprs: method
            .params
            .iter()
            .map(|(_, type_ann, _, _)| type_ann.clone())
            .chain(method.variadic.iter().map(|_| method.variadic_type.clone()))
            .collect(),
        param_attributes: method.param_attributes.clone(),
        defaults,
        return_type,
        declared_return: method.return_type.is_some(),
        by_ref_return: method.by_ref_return,
        ref_params,
        declared_params: method
            .params
            .iter()
            .map(|(_, type_ann, _, _)| type_ann.is_some())
            .chain(
                method
                    .variadic
                    .iter()
                    .map(|_| method.variadic_type.is_some()),
            )
            .collect(),
        variadic: method.variadic.clone(),
        deprecation: extract_deprecation(&method.attributes),
    });
    // A declared element type on the variadic (`int ...$xs`) constrains every collected argument.
    // `callable_wrapper_sig` defaults the variadic container to `array<mixed>`; refine it to the
    // declared element type so call validation enforces it.
    if !method.variadic_by_ref {
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
    }
    Ok(sig)
}

/// Returns `Some(reason)` when the attribute list contains a `#[\Deprecated]`
/// marker, with `reason` set to the attribute's first string argument (or an
/// empty string if absent). Match is case-insensitive on the last segment of
/// the attribute name.
pub(crate) fn extract_deprecation(groups: &[crate::parser::ast::AttributeGroup]) -> Option<String> {
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

/// Returns the number of fixed (non-variadic) parameters in `sig`.
///
/// The trailing variadic parameter, when present, is materialized as the last
/// entry of `sig.params` by `callable_wrapper_sig`; this excludes it so callers
/// can reason about the positional/fixed arity separately from the variadic
/// tail. `sig.params` always holds at least the variadic entry when
/// `sig.variadic` is set, so the subtraction never underflows.
fn fixed_param_count(sig: &FunctionSig) -> usize {
    let total = sig.params.len();
    if sig.variadic.is_some() {
        total.saturating_sub(1)
    } else {
        total
    }
}

/// Returns `true` if parameter `index` of `sig` is optional (has a default value).
///
/// An out-of-range `index` is treated as optional: there is no required
/// parameter at that position, so it cannot impose a call-site requirement.
fn param_is_optional(sig: &FunctionSig, index: usize) -> bool {
    sig.defaults.get(index).map_or(true, |d| d.is_some())
}

/// Validates that `child_sig` is a contravariant-compatible (LSP) override of
/// `parent_sig`.
///
/// PHP method-override parameters are contravariant: a child may accept *more*
/// than the parent, never fewer. This allows a child to add trailing optional
/// parameters, add a variadic, or make a required parent parameter optional,
/// while still rejecting the genuinely-incompatible cases: dropping a parameter
/// (without a covering variadic), adding a required parameter, making an
/// optional parent parameter required, removing the parent's variadic, or
/// changing the by-reference-ness of an overlapping parameter.
///
/// By-reference (`ref_params`) matching stays strict, but is compared only over
/// the overlapping prefix so an added trailing by-value parameter does not trip
/// it. Reports errors with `context` and `kind` (e.g., "overriding method").
pub(crate) fn validate_signature_compatibility(
    span: crate::span::Span,
    owner_name: &str,
    method_name: &str,
    child_sig: &FunctionSig,
    parent_sig: &FunctionSig,
    kind: &str,
    context: &str,
) -> Result<(), CompileError> {
    let parent_fixed = fixed_param_count(parent_sig);
    let child_fixed = fixed_param_count(child_sig);
    let parent_variadic = parent_sig.variadic.is_some();
    let child_variadic = child_sig.variadic.is_some();

    // Count / dropped-parameter rule: the child must accept at least as many
    // positional arguments as the parent can supply. Fewer fixed parameters is
    // only acceptable when the child has a variadic that absorbs the tail.
    if child_fixed < parent_fixed && !child_variadic {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change parameter count when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    // Added-parameter rule: every child parameter beyond the parent's fixed arity
    // must be optional (the variadic tail lives past `child_fixed`); the parent's
    // callers never supply it, so requiring it would reject legal calls.
    for index in parent_fixed..child_fixed {
        if !param_is_optional(child_sig, index) {
            return Err(CompileError::new(
                span,
                &format!(
                    "Cannot add a required parameter when {} {}: {}::{}",
                    context, kind, owner_name, method_name
                ),
            ));
        }
    }

    // Overlapping-optionality rule: an optional parent parameter must stay
    // optional in the child (`parent_optional[i]` implies `child_optional[i]`);
    // making it required rejects callers who omit it.
    let overlap = parent_fixed.min(child_fixed);
    for index in 0..overlap {
        if param_is_optional(parent_sig, index) && !param_is_optional(child_sig, index) {
            return Err(CompileError::new(
                span,
                &format!(
                    "Cannot make an optional parameter required when {} {}: {}::{}",
                    context, kind, owner_name, method_name
                ),
            ));
        }
    }

    // Variadic rule: the child may add a variadic, but may not remove the
    // parent's (`parent_variadic` implies `child_variadic`); removing it would
    // reject calls that pass extra arguments the parent accepts.
    if parent_variadic && !child_variadic {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change variadic parameter shape when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    // By-reference rule: pass-by-reference-ness must match exactly over the
    // overlapping prefix. Trailing child parameters are excluded so an added
    // optional by-value parameter does not trip the comparison.
    let ref_prefix = parent_sig.ref_params.len().min(child_sig.ref_params.len());
    if child_sig.ref_params[..ref_prefix] != parent_sig.ref_params[..ref_prefix] {
        return Err(CompileError::new(
            span,
            &format!(
                "Cannot change pass-by-reference parameters when {} {}: {}::{}",
                context, kind, owner_name, method_name
            ),
        ));
    }

    // Required-parameter-count backstop: the child may require fewer parameters
    // than the parent, never more (`child_required <= parent_required`). The
    // rules above already cover the observable cases; this guards any residual
    // mismatch defensively.
    if required_param_count(child_sig) > required_param_count(parent_sig) {
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

/// Checks a preserved late-static parent/interface return against a child declaration.
///
/// A concrete class name cannot replace `static`: that would become unsound for further
/// subclasses. `never` remains a valid covariant narrowing, while compound returns are
/// compared after binding only their `static` members to the current receiver.
pub(crate) fn late_static_return_compatible(
    checker: &Checker,
    expected: Option<&TypeExpr>,
    actual: Option<&TypeExpr>,
    actual_resolved: &PhpType,
    receiver_type: &str,
    span: crate::span::Span,
) -> Result<Option<bool>, CompileError> {
    let Some(expected) = expected else {
        return Ok(None);
    };
    if matches!(actual_resolved, PhpType::Never) {
        return Ok(Some(true));
    }
    let Some(actual) = actual.filter(|return_type| return_type.contains_late_static()) else {
        return Ok(Some(false));
    };
    let expected =
        checker.resolve_late_static_return_type_hint(expected, receiver_type, span)?;
    let actual = checker.resolve_late_static_return_type_hint(actual, receiver_type, span)?;
    Ok(Some(declared_return_type_compatible(
        checker, &expected, &actual,
    )))
}

/// Force-builds any concrete class referenced by a covariant return type — a bare
/// `Object(name)` or any `Object(name)` member of a `Union` (e.g. a nullable `?Dog`
/// return, `Union([Object("Dog"), Void])`) — that is not yet registered in
/// `checker.classes`, mirroring the identical prebuild step in
/// `schema/classes/interfaces.rs`'s interface-implementation return-type check.
///
/// The top-level class-building driver visits classes in `HashMap` iteration order, so a
/// method's declared return type may reference a class from an unrelated hierarchy that
/// has not been built yet. Without this, `Checker::is_subclass_of` (which only walks
/// `checker.classes`) nondeterministically rejects a legal covariant override depending on
/// build order (`Sub::make(): Dog` overriding `Base::make(): Animal`, and equally
/// `Sub::make(): ?Dog` overriding `Base::make(): ?Animal`, intermittently failed when `Dog`
/// had not been built yet). `class.name` itself is skipped: the class currently being
/// built is always mid-construction here, so attempting to recursively build it would
/// either no-op (already present) or spuriously trip the circular-inheritance guard for a
/// method that simply returns its own class.
fn ensure_return_type_classes_built(
    return_type: &PhpType,
    class_name: &str,
    class_map: &HashMap<String, FlattenedClass>,
    checker: &mut Checker,
    next_class_id: &mut u64,
    building: &mut HashSet<String>,
) -> Result<(), CompileError> {
    match return_type {
        PhpType::Object(actual_name) => {
            if actual_name != class_name
                && class_map.contains_key(actual_name)
                && !checker.classes.contains_key(actual_name)
            {
                super::classes::build_class_info_recursive(
                    actual_name,
                    class_map,
                    checker,
                    next_class_id,
                    building,
                )?;
            }
            Ok(())
        }
        PhpType::Union(members) => {
            for member in members {
                ensure_return_type_classes_built(
                    member,
                    class_name,
                    class_map,
                    checker,
                    next_class_id,
                    building,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Validates that `method` can override `parent_sig` in class `class_name`.
/// Builds the child signature via `build_method_sig`, skips validation for `__construct`,
/// checks signature compatibility, and ensures the child does not remove a declared
/// return type when the parent has one or make it incompatible.
///
/// `class_map`/`next_class_id`/`building` let this force-build a covariant return type's
/// class(es) on demand — see [`ensure_return_type_classes_built`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_override_signature(
    checker: &mut Checker,
    class_map: &HashMap<String, FlattenedClass>,
    next_class_id: &mut u64,
    building: &mut HashSet<String>,
    class: &crate::types::traits::FlattenedClass,
    method: &ClassMethod,
    parent_sig: &FunctionSig,
    parent_late_static_return: Option<&TypeExpr>,
    is_static: bool,
) -> Result<(), CompileError> {
    let kind = if is_static { "static method" } else { "method" };
    let class_name = class.name.as_str();
    let child_sig = build_method_sig(checker, method, class_name)?;
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
    ensure_return_type_classes_built(
        &child_sig.return_type,
        &class.name,
        class_map,
        checker,
        next_class_id,
        building,
    )?;
    let late_static_compatible = late_static_return_compatible(
        checker,
        parent_late_static_return,
        method.return_type.as_ref(),
        &child_sig.return_type,
        class_name,
        method.span,
    )?;
    let return_compatible = late_static_compatible.unwrap_or_else(|| {
        declared_return_type_compatible(
            checker,
            &parent_sig.return_type,
            &child_sig.return_type,
        ) || covariant_self_return_compatible(
            checker,
            class_name,
            class.extends.as_deref(),
            &class.implements,
            parent_sig,
            &child_sig,
        )
    });
    if parent_sig.declared_return && !return_compatible {
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

/// Returns true when a child method may return the child class itself against a wider parent return.
///
/// Covers PHP covariant returns (`parent::w(): Base` overridden as `w(): static` / `w(): Child`)
/// while the child is mid-construction — `type_accepts` cannot see the subclass edge yet because
/// `checker.classes` lacks the child, but the parent class is already registered.
fn covariant_self_return_compatible(
    checker: &Checker,
    class_name: &str,
    extends: Option<&str>,
    implements: &[String],
    parent_sig: &FunctionSig,
    child_sig: &FunctionSig,
) -> bool {
    match (&parent_sig.return_type, &child_sig.return_type) {
        (PhpType::Object(expected_name), PhpType::Object(actual_name))
            if actual_name == class_name =>
        {
            if expected_name == class_name {
                return true;
            }
            if let Some(parent) = extends {
                if parent == expected_name
                    || checker.is_subclass_of(parent, expected_name)
                    || checker.class_implements_interface(parent, expected_name)
                {
                    return true;
                }
            }
            implements.iter().any(|iface| {
                iface == expected_name
                    || checker.interface_extends_interface(iface, expected_name)
            })
        }
        _ => false,
    }
}
