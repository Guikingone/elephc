//! Purpose:
//! Builds runtime-visible user-function signatures and callable lookup metadata.
//!
//! Called from:
//! - `super::super::finalize_user_asm()` and class metadata filtering.
//!
//! Key details:
//! - Excludes synthetic property initializers and preserves include-variant callable groups.

use super::*;

/// Returns user functions visible to runtime callable-name metadata.
pub(in crate::codegen) fn runtime_user_function_sigs(module: &Module) -> HashMap<String, FunctionSig> {
    let mut functions = module
        .functions
        .iter()
        .filter(|function| !is_property_init_thunk_function(function))
        .map(|function| (function.name.clone(), ir_function_sig(function)))
        .collect::<HashMap<_, _>>();
    for group in function_variants::collect_dispatch_groups(module) {
        if let Some(function) = function_variants::variant_callee_for_group(module, &group.name) {
            functions
                .entry(group.name.clone())
                .or_insert_with(|| ir_function_sig(function));
        }
    }
    functions
}

/// Returns true for synthetic property-default init thunks, which are not PHP callables.
pub(in crate::codegen) fn is_property_init_thunk_function(function: &Function) -> bool {
    function.name.starts_with("_class_propinit_")
}

/// Reconstructs callable metadata from an EIR function when no source signature is attached.
pub(in crate::codegen) fn ir_function_sig(function: &Function) -> FunctionSig {
    if let Some(signature) = &function.signature {
        return signature.clone();
    }
    FunctionSig {
        params: function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.php_type.clone()))
            .collect(),
        param_type_exprs: vec![None; function.params.len()],
        param_attributes: vec![Vec::new(); function.params.len()],
        defaults: vec![None; function.params.len()],
        return_type: function.return_php_type.clone(),
        declared_return: false,
        by_ref_return: function.flags.by_ref_return,
        ref_params: function.params.iter().map(|param| param.by_ref).collect(),
        declared_params: vec![true; function.params.len()],
        variadic: function
            .params
            .iter()
            .find(|param| param.variadic)
            .map(|param| param.name.clone()),
        deprecation: None,
        is_generator: false,
    }
}

/// Returns include-variant public names that runtime callable lookup must check dynamically.
pub(in crate::codegen) fn runtime_function_variant_groups(module: &Module) -> HashSet<String> {
    function_variants::collect_dispatch_groups(module)
        .into_iter()
        .map(|group| group.name)
        .collect()
}

/// Returns true when runtime callable helpers need broad user callable metadata.
pub(in crate::codegen) fn module_uses_dynamic_callable_lookup(module: &Module) -> bool {
    module
        .functions
        .iter()
        .chain(module.class_methods.iter())
        .chain(module.closures.iter())
        .chain(module.fiber_wrappers.iter())
        .chain(module.callback_wrappers.iter())
        .chain(module.extern_callback_trampolines.iter())
        .chain(module.runtime_callable_invokers.iter())
        .any(function_uses_dynamic_callable_lookup)
}

/// Returns true when one function calls `is_callable()` on a runtime-shaped value.
pub(in crate::codegen) fn function_uses_dynamic_callable_lookup(function: &Function) -> bool {
    function.instructions.iter().any(|inst| {
        if !is_dynamic_callable_lookup_builtin(inst) || inst.operands.is_empty() {
            return false;
        }
        let Some(value) = function.value(inst.operands[0]) else {
            return false;
        };
        matches!(
            value.php_type.codegen_repr(),
            PhpType::Str
                | PhpType::Array(_)
                | PhpType::AssocArray { .. }
                | PhpType::Object(_)
                | PhpType::Mixed
                | PhpType::Union(_)
                | PhpType::Iterable
        )
    })
}

/// Returns true for an EIR builtin instruction that calls PHP `is_callable()`.
pub(in crate::codegen) fn is_dynamic_callable_lookup_builtin(inst: &crate::ir::Instruction) -> bool {
    typed_builtin_target(inst).is_some_and(|target| target.is_callable_lookup())
}
