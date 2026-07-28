//! Purpose:
//! Type-checks the numeric PHP builtin family.
//! Validates arity, argument types, warning-producing cases, and inferred return types for direct calls.
//!
//! Called from:
//! - `crate::types::checker::builtins::check_builtin()`
//!
//! Key details:
//! - Signatures, callable aliases, optimizer effects, and codegen builtin dispatch must remain in lockstep.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

type BuiltinResult = Result<Option<PhpType>, CompileError>;

/// Returns true when an `int`-taking builtin argument is acceptable under the gradual-typing
/// boundary model.
///
/// This is the SINGLE source of truth for "PHP would accept this value where an `int` is
/// declared". PHP's non-strict scalar coercion already turns `bool`, `float` and the `false`
/// subtype into an integer at the call boundary, and `Mixed` is the gradual top type (EIR
/// emits a runtime unbox), so all of those are accepted. A `Union` is accepted only when
/// EVERY member is itself acceptable — which is what makes elephc's own `int|float`
/// arithmetic result type (`$a + $b`) and `int|false` builtin returns usable as `int`
/// arguments without opening the door to arrays, objects, or strings.
///
/// Deliberately NOT accepted: `Str` (a non-numeric string is a genuine PHP `TypeError`),
/// `Void`/`Never` (null is only valid where a builtin declares `?int` — see
/// `crate::builtins::io::touch`, which composes this predicate with an explicit null arm),
/// and every container/resource type.
pub(crate) fn accepts_gradual_int(ty: &PhpType) -> bool {
    match ty {
        PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::False | PhpType::Mixed => true,
        PhpType::Union(members) => !members.is_empty() && members.iter().all(accepts_gradual_int),
        _ => false,
    }
}

/// Type-checks numeric and language-construct PHP builtins.
///
/// Validates argument count, argument types, and special cases (e.g., `buffer_free`
/// restriction on `$this`, locals-only) for the builtin functions in the numeric
/// family. Returns the inferred `PhpType` on success, or a `CompileError` on type/
/// arity mismatch.
///
/// ## Supported builtins
/// - Legacy scalar aliases not yet migrated into `src/builtins/`: `strval`,
///   `is_double`, `is_real`, `is_integer`, `is_long`
/// - Numeric conversions not yet migrated: `hexdec`, `bindec`
/// - Type inspection not yet migrated: `get_debug_type`
/// - Buffers: `buffer_len`, `buffer_free`
///
/// `exit`/`die`/`empty`/`unset`/`isset` are handled by `super::language_constructs` and never
/// reach this dispatcher.
///
/// ## Arguments
/// - `checker`: mutable checker state for inference
/// - `name`: lowercase builtin name (case-insensitive lookup is handled by caller)
/// - `args`: parsed argument expressions
/// - `span`: source span for error reporting
/// - `env`: current type environment
///
/// ## Returns
/// `Ok(Some(PhpType))` with the inferred return type, `Ok(None)` for unknown builtins
/// (caller falls through), or `Err(CompileError)` on validation failure.
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        // NOTE: `exit`/`die` are NOT handled here. `Checker::check_builtin` routes them to
        // `super::language_constructs::check` before this function is ever reached, so a copy
        // of the check in this module was unreachable duplicated policy. The single source of
        // truth for the `exit()` status argument is `language_constructs::check`, which
        // delegates to `accepts_gradual_int` above.
        "strval" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "strval() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "is_double" | "is_real" | "is_integer" | "is_long" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "method_exists" | "property_exists" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 2 arguments", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            checker.infer_type(&args[1], env)?;
            Ok(Some(PhpType::Bool))
        }
        "hexdec" => {
            // PHP: hexdec(string $hex_string): int|float. Non-hex characters are
            // ignored. elephc parses the hex digits into a 64-bit integer, which
            // covers the practical range (Unicode escapes, color codes, etc.).
            if args.len() != 1 {
                return Err(CompileError::new(span, "hexdec() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "bindec" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "bindec() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "get_debug_type" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "get_debug_type() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "empty" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "empty() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "unset" => {
            if args.is_empty() {
                return Err(CompileError::new(span, "unset() takes at least 1 argument"));
            }
            for arg in args {
                check_unset_arg(checker, arg, env)?;
            }
            Ok(Some(PhpType::Void))
        }
        _ => Ok(None),
    }
}

/// Infers the result type for the single-array form of `min`/`max`.
///
/// The result is the element type of the array argument: an `Int`-element array
/// yields `Int`, a `Float`-element array yields `Float`, and any other or unknown
/// element shape (including `Mixed` and empty-array sentinels) yields `Mixed` so
/// heterogeneous or gradually-typed arrays are handled at runtime. Keeps
/// `max([1, 2, 3])` an `Int` while `max([1.5, 2.5])` becomes `Float`.
pub(crate) fn min_max_single_array_result(ty: &PhpType) -> PhpType {
    let elem = match ty {
        PhpType::Array(elem) => elem.as_ref().clone(),
        PhpType::AssocArray { value, .. } => value.as_ref().clone(),
        _ => PhpType::Mixed,
    };
    match elem.codegen_repr() {
        PhpType::Int => PhpType::Int,
        PhpType::Float => PhpType::Float,
        _ => PhpType::Mixed,
    }
}


/// Type-checks one `unset()` operand while preserving PHP's non-reading property semantics.
fn check_unset_arg(checker: &mut Checker, arg: &Expr, env: &TypeEnv) -> Result<(), CompileError> {
    if let ExprKind::PropertyAccess { object, property }
    | ExprKind::NullsafePropertyAccess { object, property } = &arg.kind
    {
        let object_ty = checker.infer_type(object, env)?;
        if unset_object_property_probe_is_valid(checker, &object_ty, property, arg)? {
            return Ok(());
        }
    }
    checker.infer_type(arg, env).map(|_| ())
}

/// Returns true when `unset($object->property)` can be checked without reading the property.
fn unset_object_property_probe_is_valid(
    checker: &Checker,
    object_ty: &PhpType,
    property: &str,
    arg: &Expr,
) -> Result<bool, CompileError> {
    match object_ty {
        PhpType::Object(class_name) => {
            unset_property_probe_is_valid_on_class(checker, class_name, property, arg)
        }
        PhpType::Mixed => Ok(true),
        PhpType::Union(members) => {
            if let Some(class_name) = checker.union_single_object_class(object_ty) {
                unset_property_probe_is_valid_on_class(checker, &class_name, property, arg)
            } else {
                Ok(members.iter().any(|member| matches!(member, PhpType::Mixed)))
            }
        }
        _ => Ok(false),
    }
}

/// Checks one known receiver class for PHP `unset($object->property)` magic/no-op legality.
fn unset_property_probe_is_valid_on_class(
    checker: &Checker,
    class_name: &str,
    property: &str,
    arg: &Expr,
) -> Result<bool, CompileError> {
    if crate::types::checker::builtin_stdclass::is_stdclass(class_name) {
        return Ok(true);
    }
    let Some(class_info) = checker.classes.get(class_name) else {
        return Ok(false);
    };
    if let Some(visibility) = class_info.property_visibilities.get(property) {
        let declaring_class = class_info
            .property_declaring_classes
            .get(property)
            .map(String::as_str)
            .unwrap_or(class_name);
        if !checker.can_access_member(declaring_class, visibility) {
            if class_info.methods.contains_key("__unset") {
                return Ok(true);
            }
            return Err(CompileError::new(
                arg.span,
                &format!(
                    "Cannot access {} property: {}::{}",
                    Checker::visibility_label(visibility),
                    class_name,
                    property
                ),
            ));
        }
        return Ok(false);
    }
    Ok(true)
}
