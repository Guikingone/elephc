//! Purpose:
//! Reconciles type-checker declaration metadata after AST reachability pruning.
//! Keeps method maps and vtable layouts aligned with the declarations EIR can lower.
//!
//! Called from:
//! - `crate::optimize::reachability::prune_unreachable_declarations()`.
//!
//! Key details:
//! - Synthetic checker-only classes are preserved unless backed by a pruned source declaration.
//! - Vtable survivor order is stable and slots are compacted from zero.

use std::collections::{HashMap, HashSet};

use crate::names::php_symbol_key;
use crate::types::{CheckResult, ClassInfo};

use super::graph::{ClassKind, DeclarationIndex, Reachability};

/// Mutates checker metadata so every declaration table agrees with the pruned AST.
pub(super) fn check_result(
    check: &mut CheckResult,
    reachability: &Reachability,
    declarations: &DeclarationIndex,
    original_builtin_libraries: &HashSet<String>,
    remaining_builtin_libraries: &HashSet<String>,
) {
    let declared_extern_libraries: HashSet<String> = check
        .extern_functions
        .iter()
        .filter(|(name, _)| declarations.externs.contains(&php_symbol_key(name)))
        .filter_map(|(_, signature)| signature.library.clone())
        .collect();
    retain_free_function_metadata(check, reachability, declarations);
    retain_class_metadata(check, reachability, declarations);
    retain_extern_metadata(check, reachability, declarations);
    retain_callable_parameter_metadata(check, reachability, declarations);
    recompute_required_libraries(
        check,
        &declared_extern_libraries,
        original_builtin_libraries,
        remaining_builtin_libraries,
    );
}

/// Removes signatures and attributes for pruned source functions while preserving builtins.
fn retain_free_function_metadata(
    check: &mut CheckResult,
    reachability: &Reachability,
    declarations: &DeclarationIndex,
) {
    check.functions.retain(|name, _| {
        let key = php_symbol_key(name);
        (!declarations.functions.contains_key(&key) && !declarations.externs.contains(&key))
            || reachability.functions.contains(&key)
            || reachability.externs.contains(&key)
    });
    retain_function_keyed_map(
        &mut check.function_attribute_names,
        &reachability.functions,
        &declarations.functions,
    );
    retain_function_keyed_map(
        &mut check.function_attribute_args,
        &reachability.functions,
        &declarations.functions,
    );
    retain_function_keyed_map(
        &mut check.callable_return_sigs,
        &reachability.functions,
        &declarations.functions,
    );
    retain_function_keyed_map(
        &mut check.callable_array_return_sigs,
        &reachability.functions,
        &declarations.functions,
    );
}

/// Filters a function-keyed checker map only when its key belongs to a source declaration.
fn retain_function_keyed_map<T>(
    map: &mut HashMap<String, T>,
    reachable: &HashSet<String>,
    declared: &HashMap<String, super::usage::Usage>,
) {
    map.retain(|name, _| {
        let key = php_symbol_key(name);
        !declared.contains_key(&key) || reachable.contains(&key)
    });
}

/// Removes pruned class-like schemas and filters method metadata on live source classes.
fn retain_class_metadata(
    check: &mut CheckResult,
    reachability: &Reachability,
    declarations: &DeclarationIndex,
) {
    check.classes.retain(|name, _| {
        let key = php_symbol_key(name);
        !declarations.classes.contains_key(&key) || reachability.classes.contains(&key)
    });
    for (name, class_info) in &mut check.classes {
        let key = php_symbol_key(name);
        if declarations.classes.contains_key(&key) {
            prune_class_methods(&key, class_info, reachability);
        }
    }
    check.interfaces.retain(|name, _| {
        let key = php_symbol_key(name);
        !matches!(
            declarations.classes.get(&key).map(|node| node.kind),
            Some(ClassKind::Interface)
        ) || reachability.classes.contains(&key)
    });
    check.enums.retain(|name, _| {
        let key = php_symbol_key(name);
        !matches!(
            declarations.classes.get(&key).map(|node| node.kind),
            Some(ClassKind::Enum)
        ) || reachability.classes.contains(&key)
    });
    check.packed_classes.retain(|name, _| {
        let key = php_symbol_key(name);
        !declarations.packed_classes.contains(&key) || reachability.classes.contains(&key)
    });
}

/// Filters every instance/static method map and rebuilds stable compact vtable slots.
fn prune_class_methods(
    class_key: &str,
    info: &mut ClassInfo,
    reachability: &Reachability,
) {
    let instance_impl = info.method_impl_classes.clone();
    let instance_declaring = info.method_declaring_classes.clone();
    let static_impl = info.static_method_impl_classes.clone();
    let static_declaring = info.static_method_declaring_classes.clone();
    let keep_instance: HashSet<String> = info
        .methods
        .keys()
        .filter(|method| {
            method_is_live(
                class_key,
                method,
                false,
                &instance_impl,
                &instance_declaring,
                reachability,
            )
        })
        .cloned()
        .collect();
    let keep_static: HashSet<String> = info
        .static_methods
        .keys()
        .filter(|method| {
            method_is_live(
                class_key,
                method,
                true,
                &static_impl,
                &static_declaring,
                reachability,
            )
        })
        .cloned()
        .collect();
    let keep_any: HashSet<String> = keep_instance.union(&keep_static).cloned().collect();

    info.method_decls.retain(|method| {
        let key = php_symbol_key(&method.name);
        if method.is_static {
            keep_static.contains(&key)
        } else {
            keep_instance.contains(&key)
        }
    });
    retain_keys(&mut info.methods, &keep_instance);
    retain_keys(&mut info.late_static_method_returns, &keep_instance);
    retain_keys(&mut info.callable_method_return_sigs, &keep_any);
    retain_keys(&mut info.callable_array_method_return_sigs, &keep_any);
    retain_keys(&mut info.method_visibilities, &keep_instance);
    info.final_methods.retain(|key| keep_instance.contains(key));
    retain_keys(&mut info.method_declaring_classes, &keep_instance);
    retain_keys(&mut info.method_impl_classes, &keep_instance);

    retain_keys(&mut info.static_methods, &keep_static);
    retain_keys(&mut info.late_static_static_method_returns, &keep_static);
    retain_keys(&mut info.static_method_visibilities, &keep_static);
    info.final_static_methods.retain(|key| keep_static.contains(key));
    retain_keys(&mut info.static_method_declaring_classes, &keep_static);
    retain_keys(&mut info.static_method_impl_classes, &keep_static);

    info.method_attribute_names
        .retain(|key, _| keep_any.contains(key));
    info.method_attribute_args
        .retain(|key, _| keep_any.contains(key));

    info.vtable_methods.retain(|key| keep_instance.contains(key));
    info.vtable_slots = compact_slots(&info.vtable_methods);
    info.static_vtable_methods
        .retain(|key| keep_static.contains(key));
    info.static_vtable_slots = compact_slots(&info.static_vtable_methods);
}

/// Returns whether a method is reachable through its visible, implementing, or declaring class.
fn method_is_live(
    class_key: &str,
    method: &str,
    is_static: bool,
    implementations: &HashMap<String, String>,
    declaring: &HashMap<String, String>,
    reachability: &Reachability,
) -> bool {
    let visible = (
        class_key.to_string(),
        method.to_string(),
        is_static,
    );
    if reachability.methods.contains(&visible) {
        return true;
    }
    implementations
        .get(method)
        .into_iter()
        .chain(declaring.get(method))
        .any(|owner| {
            reachability.methods.contains(&(
                php_symbol_key(owner),
                method.to_string(),
                is_static,
            ))
        })
}

/// Retains string-keyed map entries selected by one canonical method keep-set.
fn retain_keys<T>(map: &mut HashMap<String, T>, keep: &HashSet<String>) {
    map.retain(|key, _| keep.contains(key));
}

/// Rebuilds vtable slots from survivor order without sorting or leaving gaps.
fn compact_slots(methods: &[String]) -> HashMap<String, usize> {
    methods
        .iter()
        .enumerate()
        .map(|(slot, method)| (method.clone(), slot))
        .collect()
}

/// Removes pruned FFI function/class schemas while leaving globals conservative.
fn retain_extern_metadata(
    check: &mut CheckResult,
    reachability: &Reachability,
    declarations: &DeclarationIndex,
) {
    check.extern_functions.retain(|name, _| {
        let key = php_symbol_key(name);
        !declarations.externs.contains(&key) || reachability.externs.contains(&key)
    });
    check.extern_classes.retain(|name, _| {
        let key = php_symbol_key(name);
        !declarations.extern_classes.contains(&key) || reachability.classes.contains(&key)
    });
}

/// Drops callable-parameter refinements owned by pruned functions or methods.
fn retain_callable_parameter_metadata(
    check: &mut CheckResult,
    reachability: &Reachability,
    declarations: &DeclarationIndex,
) {
    check.callable_param_sigs.retain(|(owner, _), _| {
        if let Some((class, method)) = owner.rsplit_once("::") {
            let class_key = php_symbol_key(class);
            let method_key = php_symbol_key(method);
            if declarations.classes.contains_key(&class_key) {
                return reachability
                    .methods
                    .iter()
                    .any(|(kept_class, kept_method, _)| {
                        kept_class == &class_key && kept_method == &method_key
                    });
            }
            return true;
        }
        let key = php_symbol_key(owner);
        !declarations.functions.contains_key(&key) || reachability.functions.contains(&key)
    });
}

/// Removes libraries contributed exclusively by declarations and builtin calls that no longer survive.
fn recompute_required_libraries(
    check: &mut CheckResult,
    declared_extern_libraries: &HashSet<String>,
    original_builtin_libraries: &HashSet<String>,
    remaining_builtin_libraries: &HashSet<String>,
) {
    let remaining_extern_libraries: HashSet<String> = check
        .extern_functions
        .values()
        .filter_map(|signature| signature.library.clone())
        .collect();
    check.required_libraries.retain(|library| {
        let removed_builtin_requirement = original_builtin_libraries.contains(library)
            && !remaining_builtin_libraries.contains(library)
            && !remaining_extern_libraries.contains(library);
        let removed_extern_requirement = declared_extern_libraries.contains(library)
            && !remaining_extern_libraries.contains(library)
            && !remaining_builtin_libraries.contains(library);
        !removed_builtin_requirement && !removed_extern_requirement
    });
}
