//! Purpose:
//! Shared evaluated-expression helpers for operators, class names, closures, and match.
//!
//! Called from:
//! - `crate::interpreter::expressions::eval_expr()`.
//!
//! Key details:
//! - Helpers operate after the parent evaluator has selected the expression shape
//!   and preserve PHP runtime conversion, class-alias, and closure-capture rules.

use super::*;
use crate::interpreter::builtins::spl::array_iterator::eval_array_iterator_new;

/// Applies one already-evaluated binary operation with eval runtime semantics.
pub(in crate::interpreter) fn eval_binary_result(
    op: EvalBinOp,
    left: RuntimeCellHandle,
    right: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match op {
        EvalBinOp::Add => {
            let left_tag = values.type_tag(left)?;
            let right_tag = values.type_tag(right)?;
            if matches!(left_tag, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC)
                && matches!(right_tag, EVAL_TAG_ARRAY | EVAL_TAG_ASSOC)
            {
                eval_array_union_result(left, right, values)
            } else {
                values.add(left, right)
            }
        }
        EvalBinOp::Sub => values.sub(left, right),
        EvalBinOp::Mul => values.mul(left, right),
        EvalBinOp::Div => values.div(left, right),
        EvalBinOp::Mod => values.modulo(left, right),
        EvalBinOp::Pow => values.pow(left, right),
        EvalBinOp::BitAnd
        | EvalBinOp::BitOr
        | EvalBinOp::BitXor
        | EvalBinOp::ShiftLeft
        | EvalBinOp::ShiftRight => values.bitwise(op, left, right),
        EvalBinOp::Concat => {
            let left = eval_string_context_value(left, context, values)?;
            let right = eval_string_context_value(right, context, values)?;
            values.concat(left, right)
        }
        EvalBinOp::LogicalXor => {
            let left_truthy = values.truthy(left)?;
            let right_truthy = values.truthy(right)?;
            values.bool_value(left_truthy ^ right_truthy)
        }
        EvalBinOp::LooseEq
        | EvalBinOp::LooseNotEq
        | EvalBinOp::StrictEq
        | EvalBinOp::StrictNotEq
        | EvalBinOp::Lt
        | EvalBinOp::LtEq
        | EvalBinOp::Gt
        | EvalBinOp::GtEq => values.compare(op, left, right),
        EvalBinOp::Spaceship => values.spaceship(left, right),
        EvalBinOp::LogicalAnd | EvalBinOp::LogicalOr => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Builds PHP's left-biased array union while preserving insertion order.
fn eval_array_union_result(
    left: RuntimeCellHandle,
    right: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut result = values.array_clone_shallow(left)?;
    let right_len = values.array_len(right)?;
    for position in 0..right_len {
        let key = values.array_iter_key(right, position)?;
        let exists = values.array_key_exists(key, result)?;
        if values.truthy(exists)? {
            continue;
        }
        let value = values.array_get(right, key)?;
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}

/// Evaluates a runtime property or method name expression and returns its PHP string bytes as UTF-8.
pub(in crate::interpreter) fn eval_dynamic_member_name(
    expr: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let value = eval_expr(expr, context, scope, values)?;
    let value = eval_string_context_value(value, context, values)?;
    let bytes = values.string_bytes(value)?;
    String::from_utf8(bytes).map_err(|_| EvalStatus::RuntimeFatal)
}

/// Reads an array element or dispatches `ArrayAccess::offsetGet()` for objects.
pub(in crate::interpreter) fn eval_array_get_result(
    array: RuntimeCellHandle,
    index: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.type_tag(array)? != EVAL_TAG_OBJECT {
        let array_identity = values.raw_value_word(array)?;
        let key = eval_array_reference_key(index, values)?;
        let target = key.as_ref().and_then(|key| {
            context
                .array_element_alias(array_identity, &key)
                .cloned()
        });
        if crate::eval_trace::enabled() {
            eprintln!(
                "[elephc-eval-trace] phase=array_reference_read identity={array_identity:#x} key={key:?} bound={}",
                target.is_some(),
            );
        }
        if let Some(target) = target {
            return eval_reference_target_value(&target, context, values);
        }
        return values.array_get(array, index);
    }
    if !eval_array_access_object_matches(array, context, values)? {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_method_call_result(array, "offsetGet", vec![index], context, values)
}

/// Returns whether an object value satisfies PHP's `ArrayAccess` interface.
pub(in crate::interpreter) fn eval_array_access_object_matches(
    value: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    dynamic_object_is_a(value, "ArrayAccess", false, context, values)?
        .map_or_else(|| values.object_is_a(value, "ArrayAccess", false), Ok)
}

/// Evaluates one PHP scalar cast expression through the runtime conversion hooks.
pub(super) fn eval_cast_expr(
    target: &EvalCastType,
    expr: &EvalExpr,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let value = eval_expr(expr, context, scope, values)?;
    match target {
        EvalCastType::Int => values.cast_int(value),
        EvalCastType::Float => values.cast_float(value),
        EvalCastType::String => {
            let value = eval_string_context_value(value, context, values)?;
            values.cast_string(value)
        }
        EvalCastType::Bool => values.cast_bool(value),
        EvalCastType::Array => eval_array_cast_value(value, context, values),
        EvalCastType::Object => eval_object_cast_value(value, context, values),
    }
}

/// Casts one dynamic eval value to a `stdClass` object with PHP's `(object)` rules.
///
/// PHP does not build a new object for every operand. An object casts to ITSELF -- `(object) $p`
/// is the same instance, `===` to `$p` -- so this retains rather than copies. An array becomes a
/// `stdClass` whose property names are the array's keys, integer keys included (`(object) [10, 20]`
/// has properties `"0"` and `"1"`). Null becomes an EMPTY `stdClass`, and every other scalar
/// becomes a `stdClass` with the single property `scalar`, which is the name PHP itself picks.
fn eval_object_cast_value(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match values.type_tag(value)? {
        EVAL_TAG_OBJECT => values.retain(value),
        EVAL_TAG_NULL => values.new_object("stdClass"),
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => {
            let object = values.new_object("stdClass")?;
            let len = values.array_len(value)?;
            for position in 0..len {
                // Both handles below are OWNED: `array_iter_key` allocates the key and
                // `array_get` boxes the element. The property write retains what it keeps, so
                // this frame still owes a release on each one.
                let key = values.array_iter_key(value, position)?;
                let name = values.string_bytes(key)?;
                let element = values.array_get(value, key)?;
                values.release(key)?;
                let name = String::from_utf8(name).map_err(|_| EvalStatus::RuntimeFatal)?;
                let stored =
                    eval_property_set_result(object, &name, element, context, values);
                values.release(element)?;
                stored?;
            }
            Ok(object)
        }
        _ => {
            let object = values.new_object("stdClass")?;
            eval_property_set_result(object, "scalar", value, context, values)?;
            Ok(object)
        }
    }
}

/// Casts one dynamic eval value to a PHP array while preserving array ownership and keys.
fn eval_array_cast_value(
    value: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match values.type_tag(value)? {
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => values.retain(value),
        EVAL_TAG_NULL => values.array_new(0),
        EVAL_TAG_OBJECT => eval_object_array_cast_value(value, context, values),
        _ => {
            let array = values.array_new(1)?;
            let key = values.int(0)?;
            let result = values.array_set(array, key, value);
            values.release(key)?;
            result
        }
    }
}

/// Casts the runtime-visible public properties of one object to an associative PHP array.
pub(in crate::interpreter) fn eval_object_array_cast_value(
    object: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let property_count = values.object_property_len(object)?;
    let identity = values.object_identity(object)?;
    let dynamic_properties = context.dynamic_property_values_for_clone(identity);
    if crate::eval_trace::enabled() {
        eprintln!(
            "[elephc-eval-trace] phase=array_cast_object identity={identity:#x} runtime_properties={property_count} dynamic_properties={}",
            dynamic_properties.len(),
        );
    }
    let mut emitted_properties = std::collections::HashSet::new();
    let mut result = values.assoc_new(property_count + dynamic_properties.len())?;
    for (property, value) in dynamic_properties {
        let key = values.string(&property)?;
        result = values.array_set(result, key, value)?;
        emitted_properties.insert(property);
    }
    for position in 0..property_count {
        let key = values.object_property_iter_key(object, position)?;
        let property_bytes = values.string_bytes(key)?;
        values.release(key)?;
        let property = String::from_utf8(property_bytes)
            .map_err(|_| EvalStatus::RuntimeFatal)?;
        if !emitted_properties.insert(property.clone()) {
            continue;
        }
        let value = values.property_get(object, &property)?;
        let key = values.string(&property)?;
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}

/// Constructs an object after the target class name and constructor arguments have been evaluated.
pub(in crate::interpreter) fn eval_new_object_result(
    class_name: &str,
    args: Vec<EvaluatedCallArg>,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if class_name.trim_start_matches('\\').eq_ignore_ascii_case("ArrayIterator") {
        return eval_array_iterator_new(args, context, values);
    }
    let reflection = eval_reflection_owner_new_object(class_name, args.clone(), context, values)
        .map_err(|status| trace_new_object_error("reflection", class_name, status, context))?;
    if let Some(object) = reflection {
        return Ok(object);
    }
    if let Some(class) = context.class(class_name).cloned() {
        return eval_dynamic_class_new_object(&class, args, context, scope, values)
            .map_err(|status| trace_new_object_error("eval_class", class_name, status, context));
    }
    // `CURLFile`/`CURLStringFile` DELIBERATELY FALL THROUGH to the native-class fallback
    // below, and that is the whole point rather than an oversight. They used to be
    // intercepted here alongside the curl multi/share interfaces, on a "TWO DISTINCT OBJECT
    // SPACES" argument that does not apply to them: unlike every other curl class, they are
    // PURE PHP DATA CLASSES wrapping no native handle at all
    // (`crate::curl_prelude`'s own comment above their declarations), so the real AOT class
    // an `elephc_curl`-linked program already carries works correctly inside `eval()` — its
    // properties read, its getters and setters dispatch, and
    // `crate::interpreter::builtins::curl::multipart` consumes exactly such an object when
    // it walks a `CURLOPT_POSTFIELDS` array. The interception existed only because that
    // walk did not, which made a constructible `CURLFile` a value nothing could use.
    // `new` IS AN AUTOLOAD TRIGGER IN PHP, and this path never was one. Nothing above consults a
    // loader: the reflection owners are handled first, then classes this context already declares,
    // and then allocation was attempted straight away. So a class only a registered autoloader
    // could supply reported `could not construct class "X"` — a FATAL — where PHP loads it and
    // constructs the object. `class_exists` already had the chain (see `builtins::symbols::
    // class_exists`, which calls `eval_spl_autoload_class` after its declared-check misses); this
    // is the same call, on the same terms, at the other place PHP resolves a class name.
    //
    // THE CHAIN IS AN OPPORTUNITY, NOT A GATE, and getting that backwards is a real defect I made
    // and had to measure my way out of. A first cut treated a chain MISS as the decision — no
    // loader supplied the name, therefore the class does not exist — and that is wrong, because
    // the RUNTIME has not been asked yet: `values.new_object` consults the generated AOT name
    // table, which is a different table from the one `eval_autoload_target_exists` reads. Twelve
    // interpreter tests said so immediately, `new Box()` among them, failing `UncaughtThrowable`
    // where they had returned an object; measured green on the same tree with the magician
    // reverted, so the attribution was not a guess. The boolean is therefore DISCARDED here: the
    // chain runs for its side effect — it may declare the class — and the verdict is left to the
    // runtime below.
    //
    // AND PHP NEVER AUTOLOADS AN INTERNAL CLASS. `new Exception(...)` consults no loader in PHP,
    // because the name is already a class; only a name the engine does not know starts the chain.
    // Skipping the catalog here is therefore php-correct rather than an optimisation, and it is
    // also what keeps this call free of observable side effects for the overwhelmingly common
    // case: running the chain for `Exception` allocated and released cells of its own, which
    // reordered the interpreter's release log and broke two tests asserting that the FIRST
    // released handle is the thrown object (`execute_program_finally_return_overrides_uncaught_
    // throw` and `execute_program_catches_throwable_without_variable_inside_eval`, both seeing
    // tag 1 where they expect tag 6).
    if !eval_reflection_class_like_is_internal(class_name) {
        let _ = eval_spl_autoload_class(class_name, context, values)?;
        // A LOADER USUALLY DECLARES THE CLASS INTO THIS CONTEXT rather than into the AOT tables —
        // it runs `eval(...)` or includes a file, and either way the result is an interpreter
        // class. The `context.class(...)` branch above ran BEFORE the loader did, so it has to be
        // asked again; without this the freshly loaded class falls through to `new_object`, which
        // only knows the AOT name table, and construction fails with the class sitting right
        // there.
        if let Some(class) = context.class(class_name).cloned() {
            return eval_dynamic_class_new_object(&class, args, context, scope, values).map_err(
                |status| trace_new_object_error("eval_class", class_name, status, context),
            );
        }
    }
    // THE RUNTIME HAS NOW BEEN ASKED, so a failure here is the last word and can be classified.
    // Two very different things used to share one wording, `could not construct class "X"`.
    //
    // A name PHP itself provides as a builtin class — a Throwable, an SPL container, Fiber, Phar,
    // stdClass — that reaches this point is a GAP IN THIS BUILD, not a user error: PHP would have
    // constructed the object, so blaming the program with `Class "X" not found` would read as an
    // ordinary PHP error and a missing pay-for-use gate would look like working software.
    // Measured: a runtime-included `new Fiber(function () { Fiber::suspend("s"); })` reports this,
    // where `php -n` 8.5.6 prints `s;done`, because `program_may_reference_fiber` does not read
    // `usage.includes_runtime_php` and the family is never registered.
    //
    // Any OTHER name is genuinely undefined, and PHP's answer is a CATCHABLE `Error` reading
    // `Class "X" not found` — the thing a surrounding `catch (\Throwable $e)` is written to
    // receive, and the reason the checker may now defer such a name instead of refusing it.
    //
    // The catalog is `reflection::class_lookup::eval_reflection_class_like_is_internal`, the same
    // predicate `ReflectionClass::isInternal()` answers from, rather than a second list free to
    // drift from it.
    let object = match values.new_object(class_name) {
        Ok(object) => object,
        Err(status) => {
            if eval_reflection_class_like_is_internal(class_name) {
                note_eval_runtime_failure(
                    format!("builtin class \"{class_name}\" is not available in this build"),
                    context,
                );
                return Err(trace_new_object_error(
                    "builtin_not_available",
                    class_name,
                    status,
                    context,
                ));
            }
            trace_new_object_error("allocation", class_name, status, context);
            return eval_throw_class_not_found_error(class_name, context, values);
        }
    };
    if let Err(err) = initialize_native_throwable_source(object, context, values)
        .and_then(|()| eval_native_constructor_with_evaluated_args(class_name, object, args, context, values))
    {
        trace_new_object_error("native_constructor", class_name, err, context);
        let _ = values.release(object);
        return Err(err);
    }
    Ok(object)
}

/// PHP captures Throwable source properties at allocation, before user constructors.
fn initialize_native_throwable_source(
    object: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if !values.object_is_a(object, "Throwable", false)? {
        return Ok(());
    }
    let base = if values.object_is_a(object, "Exception", false)? { "Exception" } else { "Error" };
    context.push_class_scope(base);
    let result = (|| {
        let file = values.string(&context.eval_file_magic())?;
        let write = values.property_set(object, "file", file);
        let release = values.release(file);
        write?;
        release?;
        let line = values.int(context.call_line())?;
        let write = values.property_set(object, "line", line);
        let release = values.release(line);
        write?;
        release
    })();
    context.pop_class_scope();
    result
}

/// Emits the failing AOT/eval construction substage under opt-in runtime tracing.
pub(super) fn trace_new_object_error(
    stage: &str,
    class_name: &str,
    status: EvalStatus,
    context: &ElephcEvalContext,
) -> EvalStatus {
    if crate::eval_trace::enabled() {
        let call_site = context.call_site();
        eprintln!(
            "[elephc-eval-trace] phase=new_object_error stage={stage} class={class_name:?} status={status:?} file={:?} line={}",
            call_site.0,
            call_site.2,
        );
    }
    status
}

/// Resolves special class names used by `new` while preserving AOT fallback names.
pub(super) fn eval_new_object_class_name(
    class_name: &str,
    context: &ElephcEvalContext,
) -> Result<String, EvalStatus> {
    match class_name.to_ascii_lowercase().as_str() {
        "self" | "parent" | "static" => resolve_eval_static_class_name(class_name, context),
        _ => Ok(context
            .resolve_class_name(class_name)
            .unwrap_or_else(|| class_name.trim_start_matches('\\').to_string())),
    }
}

/// Resolves a runtime class-name value used by dynamic class operations.
pub(in crate::interpreter) fn eval_dynamic_class_name(
    class_name: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    match values.type_tag(class_name)? {
        EVAL_TAG_STRING => {
            let bytes = values.string_bytes(class_name)?;
            let class_name = String::from_utf8(bytes).map_err(|_| EvalStatus::RuntimeFatal)?;
            Ok(context
                .resolve_class_like_name(&class_name)
                .unwrap_or_else(|| class_name.trim_start_matches('\\').to_string()))
        }
        EVAL_TAG_OBJECT => eval_instanceof_object_target_name(class_name, context, values),
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Returns the runtime class name for `$object::class` and rejects non-object dynamic receivers.
pub(super) fn eval_dynamic_class_name_fetch_result(
    class_name: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let tag = values.type_tag(class_name)?;
    if tag == EVAL_TAG_OBJECT {
        let class_name = eval_instanceof_object_target_name(class_name, context, values)?;
        return values.string(&class_name);
    }
    eval_throw_type_error(
        &format!(
            "Cannot use \"::class\" on {}",
            eval_class_name_fetch_type_error_name(tag)
        ),
        context,
        values,
    )
}

/// Returns PHP's type label for dynamic `::class` TypeError diagnostics.
fn eval_class_name_fetch_type_error_name(tag: u64) -> &'static str {
    match tag {
        EVAL_TAG_INT => "int",
        EVAL_TAG_FLOAT => "float",
        EVAL_TAG_STRING => "string",
        EVAL_TAG_BOOL => "bool",
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => "array",
        EVAL_TAG_RESOURCE => "resource",
        EVAL_TAG_NULL => "null",
        _ => "null",
    }
}

/// Evaluates PHP's `instanceof` operator over static and dynamic class targets.
pub(super) fn eval_instanceof_expr(
    value: &EvalExpr,
    target: &EvalInstanceOfTarget,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let value = eval_expr(value, context, scope, values)?;
    let result = match target {
        EvalInstanceOfTarget::ClassName(class_name) => {
            let target_class = eval_instanceof_static_target_name(class_name, context)?;
            let tag = values.type_tag(value)?;
            if tag == EVAL_TAG_CALLABLE {
                eval_callable_descriptor_is_closure(&target_class)
            } else if tag == EVAL_TAG_OBJECT {
                eval_instanceof_object_result(value, &target_class, context, values)?
            } else {
                false
            }
        }
        EvalInstanceOfTarget::Expr(target) => {
            let target = eval_expr(target, context, scope, values)?;
            let target_class = eval_instanceof_dynamic_target_name(target, context, values)?;
            let tag = values.type_tag(value)?;
            if tag == EVAL_TAG_CALLABLE {
                eval_callable_descriptor_is_closure(&target_class)
            } else if tag == EVAL_TAG_OBJECT {
                eval_instanceof_object_result(value, &target_class, context, values)?
            } else {
                false
            }
        }
    };
    values.bool_value(result)
}

/// Native first-class callable descriptors cross the eval ABI as a dedicated cell tag, but PHP
/// exposes them as Closure instances.
fn eval_callable_descriptor_is_closure(target_class: &str) -> bool {
    target_class
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("Closure")
}

/// Resolves a static `instanceof` target according to eval class aliases and scope keywords.
fn eval_instanceof_static_target_name(
    class_name: &str,
    context: &ElephcEvalContext,
) -> Result<String, EvalStatus> {
    match class_name.to_ascii_lowercase().as_str() {
        "self" | "parent" | "static" => resolve_eval_static_class_name(class_name, context),
        _ => Ok(eval_instanceof_resolved_target_name(class_name, context)),
    }
}

/// Resolves a dynamic `instanceof` target cell to the PHP class name it represents.
fn eval_instanceof_dynamic_target_name(
    target: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    match values.type_tag(target)? {
        EVAL_TAG_STRING => {
            let target = eval_instanceof_string_target_name(target, values)?;
            Ok(eval_instanceof_resolved_target_name(&target, context))
        }
        EVAL_TAG_OBJECT => eval_instanceof_object_target_name(target, context, values),
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Reads and normalizes one string-valued dynamic `instanceof` target.
fn eval_instanceof_string_target_name(
    target: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let bytes = values.string_bytes(target)?;
    let target = String::from_utf8(bytes).map_err(|_| EvalStatus::RuntimeFatal)?;
    Ok(target.trim_start_matches('\\').to_string())
}

/// Reads the runtime class of an object-valued dynamic `instanceof` target.
fn eval_instanceof_object_target_name(
    target: RuntimeCellHandle,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let identity = values.object_identity(target)?;
    if let Some((_, class)) = context.dynamic_object_declaring_class(identity) {
        return Ok(class.name().to_string());
    }
    let class_name = values.object_class_name(target)?;
    let bytes = values.string_bytes(class_name);
    values.release(class_name)?;
    let class_name = String::from_utf8(bytes?).map_err(|_| EvalStatus::RuntimeFatal)?;
    Ok(class_name.trim_start_matches('\\').to_string())
}

/// Applies eval alias resolution to a target class name without requiring it to exist.
fn eval_instanceof_resolved_target_name(target: &str, context: &ElephcEvalContext) -> String {
    context
        .resolve_class_name(target)
        .unwrap_or_else(|| target.trim_start_matches('\\').to_string())
}

/// Tests one object cell against a resolved `instanceof` target class/interface name.
fn eval_instanceof_object_result(
    value: RuntimeCellHandle,
    target_class: &str,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<bool, EvalStatus> {
    dynamic_object_is_a(value, target_class, false, context, values)?
        .map_or_else(|| values.object_is_a(value, target_class, false), Ok)
}

/// Materializes one eval closure literal as a PHP-visible `Closure` object.
pub(super) fn eval_closure_expr(
    function: &EvalFunction,
    captures: &[crate::eval_ir::EvalClosureCapture],
    is_static: bool,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut bindings = Vec::with_capacity(captures.len());
    for capture in captures {
        bindings.push(eval_closure_capture(capture, context, scope, values)?);
    }
    let mut closure = EvalClosure::new(function.clone(), bindings, is_static);
    closure.set_declaring_class_scopes(
        context.current_class_scope().map(str::to_string),
        context.current_called_class_scope().map(str::to_string),
    );
    // PHP binds `$this` into a non-static closure or arrow function automatically when one is
    // declared inside a method body while an instance is in scope -- this is not a `use (...)`
    // capture (and an arrow function's implicit capture list explicitly excludes `this`), so it
    // has to be captured here, at the declaration site, the same way the class scopes above are.
    // A `static function`/`static fn` never gets `$this`, even if one happens to be in scope
    // (e.g. a static closure declared inside another closure that itself captured `$this`).
    if !is_static {
        if let Some(this_value) = visible_scope_cell(context, scope, "this") {
            let retained = values.retain(this_value)?;
            closure.set_declaring_this(Some(retained));
        }
    }
    closure.set_declaring_call_site(context.call_site());
    if crate::eval_trace::enabled() {
        eprintln!(
            "[elephc-eval-trace] phase=closure_create function={:?} lexical_class={:?} called_class={:?}",
            function.name(),
            closure.declaring_class_scope(),
            closure.declaring_called_class_scope(),
        );
    }
    let name = context.define_closure(closure);
    eval_closure_object_expr(EvalClosureObjectTarget::Named(name), context, values)
}

/// Materializes one PHP-visible `Closure` object for an eval callable target.
pub(in crate::interpreter) fn eval_closure_object_expr(
    mut target: EvalClosureObjectTarget,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let object = values.new_object("stdClass")?;
    let identity = match values.object_identity(object) {
        Ok(identity) => identity,
        Err(status) => {
            values.release(object)?;
            return Err(status);
        }
    };
    if let Some(receiver) = target.receiver_mut() {
        match values.retain(*receiver) {
            Ok(retained) => *receiver = retained,
            Err(status) => {
                values.release(object)?;
                return Err(status);
            }
        }
    }
    context.register_closure_object_target(identity, target);
    Ok(object)
}

/// Evaluates one closure capture from the defining scope.
fn eval_closure_capture(
    capture: &crate::eval_ir::EvalClosureCapture,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<EvalClosureCaptureBinding, EvalStatus> {
    if capture.by_ref() {
        let expr = EvalExpr::LoadVar(capture.name().to_string());
        let (value, target) = eval_call_arg_value(&expr, context, scope, values).map_err(|status| {
            if crate::eval_trace::enabled() {
                eprintln!(
                    "[elephc-eval-trace] phase=closure_capture_error capture={:?} by_ref=true status={status:?}",
                    capture.name(),
                );
            }
            status
        })?;
        return Ok(EvalClosureCaptureBinding::new(
            capture.name(),
            value,
            target,
        ));
    }
    let value = if let Some(value) = visible_scope_cell(context, scope, capture.name()) {
        values.retain(value)?
    } else {
        values.null()?
    };
    Ok(EvalClosureCaptureBinding::new(capture.name(), value, None))
}

/// Evaluates a PHP `match` expression with strict comparison and lazy arm values.
pub(in crate::interpreter) fn eval_match_expr(
    subject: &EvalExpr,
    arms: &[EvalMatchArm],
    default: Option<&EvalExpr>,
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let subject = eval_expr(subject, context, scope, values)?;
    for arm in arms {
        for pattern in &arm.patterns {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let matched = values.compare(EvalBinOp::StrictEq, subject, pattern)?;
            if values.truthy(matched)? {
                return eval_expr(&arm.value, context, scope, values);
            }
        }
    }
    if let Some(expr) = default {
        return eval_expr(expr, context, scope, values);
    }
    eval_throw_unhandled_match_error(subject, context, values)
}

/// Raises PHP's `\UnhandledMatchError` for a `match` no arm accepted.
///
/// This used to be a bare `RuntimeFatal`, which is not the same thing at all: PHP's error is a
/// `\Error` subclass a program can CATCH, and Symfony's enum and routing code does exactly that.
/// A fatal in its place turns a handled branch into a dead process.
fn eval_throw_unhandled_match_error<T>(
    subject: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let description = eval_unhandled_match_subject(subject, values)?;
    let error = values.new_object("UnhandledMatchError")?;
    let message = values.string(&format!("Unhandled match case {description}"))?;
    let code = values.int(0)?;
    values.construct_object(error, vec![message, code])?;
    context.set_pending_throw(error);
    Err(EvalStatus::UncaughtThrowable)
}

/// Renders the unmatched subject the way PHP names it in the error message.
///
/// Measured with `php -n` 8.5.6: an int, float, bool or null is spelled out, a string is
/// single-quoted and cut to 15 characters with a trailing `...`, and anything with no readable
/// literal — an array, an object, a resource — is named by TYPE instead. A float reuses PHP's
/// own `(string)` conversion and then regains the `.0` that conversion drops, which is the one
/// difference between the two spellings: `(string)1.0` is `1` but the message says `1.0`.
fn eval_unhandled_match_subject(
    subject: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    match values.type_tag(subject)? {
        EVAL_TAG_NULL => Ok("NULL".to_string()),
        EVAL_TAG_BOOL => Ok(if values.truthy(subject)? { "true" } else { "false" }.to_string()),
        EVAL_TAG_FLOAT => {
            let text = eval_scalar_text(subject, values)?;
            let plain = !text.contains(['.', 'E', 'N', 'F']);
            Ok(if plain { format!("{text}.0") } else { text })
        }
        EVAL_TAG_INT => eval_scalar_text(subject, values),
        EVAL_TAG_STRING => {
            let text = eval_scalar_text(subject, values)?;
            let cut: String = text.chars().take(15).collect();
            Ok(if cut.chars().count() < text.chars().count() {
                format!("'{cut}...'")
            } else {
                format!("'{cut}'")
            })
        }
        EVAL_TAG_ARRAY | EVAL_TAG_ASSOC => Ok("of type array".to_string()),
        EVAL_TAG_RESOURCE => Ok("of type resource".to_string()),
        EVAL_TAG_OBJECT => {
            let class = values.object_class_name(subject)?;
            let class = eval_scalar_text(class, values)?;
            Ok(format!("of type {class}"))
        }
        _ => Ok("of type mixed".to_string()),
    }
}

/// Reads one value's PHP string conversion as UTF-8 text.
fn eval_scalar_text(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
