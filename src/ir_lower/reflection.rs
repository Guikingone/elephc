//! Purpose:
//! Selects and lowers the synthetic builtin Reflection surface reachable from EIR.
//!
//! Called from:
//! - `crate::ir_lower::program::lower()` after user functions and literal eval AOT bodies.
//!
//! Key details:
//! - Native programs lower Reflection classes to a fixed point from EIR types and calls.
//! - Eval-owned Reflection values are executed by Magician and do not root unrelated AOT bodies.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::ir::{Function, Immediate, IrType, Module, Op};
use crate::parser::ast::ExprKind;
use crate::types::{CheckResult, FunctionSig, PhpType};

use super::function;
use super::program::{
    all_lowered_functions, class_data_name, class_method_already_lowered,
    dynamic_object_new_metadata_names, include_lowered_runtime_features, php_method_key,
    string_data_name,
};

/// Builtin Reflection classes whose concrete method bodies can be lowered into EIR.
const BUILTIN_REFLECTION_CLASS_NAMES: &[&str] = &[
    "ReflectionAttribute",
    "ReflectionClass",
    "ReflectionObject",
    "ReflectionEnum",
    "ReflectionClassConstant",
    "ReflectionEnumBackedCase",
    "ReflectionEnumUnitCase",
    "ReflectionFunction",
    "ReflectionMethod",
    "ReflectionType",
    "ReflectionNamedType",
    "ReflectionParameter",
    "ReflectionProperty",
    "ReflectionUnionType",
    "ReflectionIntersectionType",
];

/// Lowers only synthetic Reflection classes reachable from native EIR.
///
/// Reflection values owned by the dynamic eval bridge are dispatched by Magician, whose
/// interpreter exposes the complete Reflection surface without forcing those bodies into AOT.
pub(super) fn lower_referenced_builtin_methods(
    module: &mut Module,
    check_result: &CheckResult,
    constants: &HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &HashMap<String, FunctionSig>,
) {
    loop {
        let methods = referenced_builtin_reflection_methods(module);
        let before = module.class_methods.len();
        for (class_name, methods) in methods {
            lower_builtin_reflection_property_init_thunk(
                &class_name,
                module,
                check_result,
                constants,
                fiber_return_sigs,
            );
            lower_builtin_reflection_class_methods(
                &class_name,
                &methods,
                module,
                check_result,
                constants,
                fiber_return_sigs,
            );
        }
        if module.class_methods.len() == before {
            break;
        }
        include_lowered_runtime_features(module);
    }
}

/// Lowers a selected Reflection class's property-default thunk once it becomes reachable.
fn lower_builtin_reflection_property_init_thunk(
    class_name: &str,
    module: &mut Module,
    check_result: &CheckResult,
    constants: &HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &HashMap<String, FunctionSig>,
) {
    let Some(class_info) = check_result.classes.get(class_name) else {
        return;
    };
    let function_name = format!("_class_propinit_{}", class_info.class_id);
    if module
        .functions
        .iter()
        .any(|function| function.name == function_name)
    {
        return;
    }
    function::lower_property_init_thunk(
        class_name,
        class_info,
        module,
        check_result,
        constants,
        fiber_return_sigs,
    );
}

/// Collects Reflection constructors and methods reachable from EIR calls and descriptors.
fn referenced_builtin_reflection_methods(module: &Module) -> BTreeMap<String, BTreeSet<String>> {
    let mut methods = BTreeMap::new();
    for function in all_lowered_functions(module) {
        for inst in &function.instructions {
            if instruction_uses_mixed_string_dispatch(function, inst) {
                collect_builtin_reflection_methods_for_receiver_type(
                    module,
                    &PhpType::Mixed,
                    "__toString",
                    &mut methods,
                );
            }
            match inst.op {
                Op::ObjectNew => {
                    if let Some(class_name) = class_data_name(module, inst) {
                        insert_builtin_reflection_method(module, class_name, "__construct", &mut methods);
                    }
                }
                Op::DynamicObjectNew => {
                    if let Some((fallback_class, required_parent)) =
                        dynamic_object_new_metadata_names(module, inst)
                    {
                        insert_builtin_reflection_method(
                            module,
                            fallback_class,
                            "__construct",
                            &mut methods,
                        );
                        insert_builtin_reflection_method(
                            module,
                            required_parent,
                            "__construct",
                            &mut methods,
                        );
                    }
                }
                Op::StaticMethodCall => {
                    if let Some((class_name, method_name)) =
                        string_data_name(module, inst).and_then(|name| name.rsplit_once("::"))
                    {
                        insert_builtin_reflection_method(module, class_name, method_name, &mut methods);
                    }
                }
                Op::MethodCall | Op::NullsafeMethodCall => {
                    collect_builtin_reflection_method_call(module, function, inst, &mut methods);
                }
                Op::FirstClassCallableNew => {
                    collect_builtin_reflection_first_class_callable(
                        module,
                        function,
                        inst,
                        &mut methods,
                    );
                }
                _ => {}
            }
        }
    }
    methods
}

/// Returns whether one EIR string context can dispatch a boxed receiver through `__toString`.
fn instruction_uses_mixed_string_dispatch(function: &Function, inst: &crate::ir::Instruction) -> bool {
    let string_context = inst.op == Op::EchoValue
        || (inst.op == Op::Cast && inst.immediate == Some(Immediate::CastTarget(IrType::Str)));
    string_context && inst.operands.first().is_some_and(|operand| {
        function.value(*operand).is_some_and(|value| {
            matches!(
                value.php_type.codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            )
        })
    })
}

/// Adds the Reflection method selected by an `object::method` first-class callable descriptor.
fn collect_builtin_reflection_first_class_callable(
    module: &Module,
    function: &Function,
    inst: &crate::ir::Instruction,
    methods: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let Some(method_name) = string_data_name(module, inst)
        .and_then(|name| name.strip_prefix("object::"))
    else {
        return;
    };
    let Some(receiver) = inst.operands.first().copied() else {
        return;
    };
    let Some(receiver_type) = function.value(receiver).map(|value| value.php_type.codegen_repr())
    else {
        return;
    };
    collect_builtin_reflection_methods_for_receiver_type(
        module,
        &receiver_type,
        method_name,
        methods,
    );
}

/// Adds the concrete Reflection implementation(s) that can service one EIR method call.
fn collect_builtin_reflection_method_call(
    module: &Module,
    function: &Function,
    inst: &crate::ir::Instruction,
    methods: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let Some(receiver) = inst.operands.first().copied() else {
        return;
    };
    let Some(receiver_type) = function.value(receiver).map(|value| value.php_type.codegen_repr())
    else {
        return;
    };
    let Some(method_name) = string_data_name(module, inst) else {
        return;
    };
    collect_builtin_reflection_methods_for_receiver_type(
        module,
        &receiver_type,
        method_name,
        methods,
    );
}

/// Adds methods for the exact or gradual Reflection receiver alternatives in one PHP type.
fn collect_builtin_reflection_methods_for_receiver_type(
    module: &Module,
    receiver_type: &PhpType,
    method_name: &str,
    methods: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match receiver_type {
        PhpType::Object(class_name) => {
            insert_builtin_reflection_method(module, class_name, method_name, methods);
        }
        PhpType::Union(members) => {
            for member in members {
                collect_builtin_reflection_methods_for_receiver_type(
                    module,
                    member,
                    method_name,
                    methods,
                );
            }
        }
        PhpType::Mixed => {
            let method_key = php_method_key(method_name);
            let candidates = module
                .class_infos
                .iter()
                .filter(|(_, class_info)| class_info.methods.contains_key(&method_key))
                .map(|(class_name, _)| class_name.clone())
                .collect::<Vec<_>>();
            for class_name in candidates {
                insert_builtin_reflection_method(module, &class_name, method_name, methods);
            }
        }
        PhpType::Array(_)
        | PhpType::AssocArray { .. }
        | PhpType::Buffer(_)
        | PhpType::Int
        | PhpType::Float
        | PhpType::Str
        | PhpType::Bool
        | PhpType::False
        | PhpType::Void
        | PhpType::Never
        | PhpType::Iterable
        | PhpType::Callable
        | PhpType::Packed(_)
        | PhpType::Pointer(_)
        | PhpType::Resource(_)
        | PhpType::TaggedScalar => {}
    }
}

/// Inserts the owning builtin Reflection implementation for one reachable method.
fn insert_builtin_reflection_method(
    module: &Module,
    class_name: &str,
    method_name: &str,
    methods: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let Some(canonical) = canonical_builtin_reflection_class_name(class_name) else {
        return;
    };
    let Some(class_info) = module.class_infos.get(canonical) else {
        return;
    };
    let method_key = php_method_key(method_name);
    let implementation = class_info
        .method_impl_classes
        .get(&method_key)
        .or_else(|| class_info.static_method_impl_classes.get(&method_key))
        .map(String::as_str)
        .unwrap_or(canonical);
    let Some(owner) = canonical_builtin_reflection_class_name(implementation) else {
        return;
    };
    methods
        .entry(owner.to_string())
        .or_default()
        .insert(method_key);
}

/// Resolves a class spelling to the canonical builtin Reflection name.
pub(super) fn canonical_builtin_reflection_class_name(class_name: &str) -> Option<&'static str> {
    let key = crate::names::php_symbol_key(class_name.trim_start_matches('\\'));
    BUILTIN_REFLECTION_CLASS_NAMES
        .iter()
        .copied()
        .find(|candidate| crate::names::php_symbol_key(candidate) == key)
}

/// Lowers all concrete synthetic methods for one builtin reflection class.
fn lower_builtin_reflection_class_methods(
    class_name: &str,
    reachable_methods: &BTreeSet<String>,
    module: &mut Module,
    check_result: &CheckResult,
    constants: &HashMap<String, (ExprKind, PhpType)>,
    fiber_return_sigs: &HashMap<String, FunctionSig>,
) {
    let Some(class_info) = check_result.classes.get(class_name) else {
        return;
    };
    let before = module.class_methods.len();
    for method in &class_info.method_decls {
        if !method.has_body {
            continue;
        }
        let generated_body;
        let method_key = crate::names::php_symbol_key(&method.name);
        if !reachable_methods.contains(&method_key) {
            continue;
        }
        if class_method_already_lowered(module, class_name, &method_key, method.is_static) {
            continue;
        }
        let body = if class_name == "ReflectionAttribute" && method_key == "newinstance" {
            let function_attrs = function_attribute_sources(module);
            generated_body =
                crate::codegen::reflection::build_attribute_new_instance_body_with_extra(
                    &check_result.classes,
                    &function_attrs,
                );
            generated_body.as_slice()
        } else if class_name == "ReflectionAttribute" && method_key == "getarguments" {
            // Materialize captured attribute arguments through the normal array
            // lowering (named arguments and associative arrays included) rather
            // than a bespoke codegen path.
            let function_attrs = function_attribute_sources(module);
            generated_body = crate::codegen::reflection::build_attribute_get_arguments_body_with_extra(
                &check_result.classes,
                &function_attrs,
            );
            generated_body.as_slice()
        } else {
            &method.body
        };
        function::lower_class_method(
            class_name,
            &method.name,
            method.is_static,
            &method.params,
            method.return_type.as_ref(),
            body,
            module,
            check_result,
            constants,
            fiber_return_sigs,
        );
    }
    for method in module.class_methods.iter_mut().skip(before) {
        method.flags.is_synthetic = true;
    }
}

/// Returns reflection-visible top-level function attribute metadata sources.
fn function_attribute_sources(
    module: &Module,
) -> Vec<crate::codegen::reflection::AttributeMetadataSource<'_>> {
    module
        .functions
        .iter()
        .filter(|function| !function.attribute_names.is_empty())
        .map(|function| {
            (
                function.attribute_names.as_slice(),
                function.attribute_args.as_slice(),
            )
        })
        .collect()
}
