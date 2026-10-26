//! Purpose:
//! Applies post-injection checker signature refinements for SPL iterator storage metadata.
//! Keeps concrete PhpType patches separate from class-shape construction.
//!
//! Called from:
//! - `super::patch_builtin_spl_storage_signatures()`.
//!
//! Key details:
//! - Some synthetic class properties need precise runtime PhpType metadata after flattening.
//! - Object and callable storage types are tightened here without changing source-level declarations.

use crate::names::php_symbol_key;
use crate::types::PhpType;

use super::common::{int_expr, null_expr};
use super::containers::{DLL_MODE_PROPERTY, DLL_STORAGE_PROPERTY};
use super::super::Checker;

/// Patches builtin SPL storage signatures in the compiler metadata registry.
pub(super) fn patch_builtin_spl_storage_signatures(checker: &mut Checker) {
    patch_dll_storage(checker);
    patch_object_storage_serialization(checker);
    super::native_property_storage::box_heap_user_property_storage(checker);
    let return_type = PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    };
    let method_key = php_symbol_key("getArrayCopy");
    for class_name in ["ArrayIterator", "ArrayObject"] {
        if let Some(class_info) = checker.classes.get_mut(class_name) {
            if let Some(sig) = class_info.methods.get_mut(&method_key) {
                sig.return_type = return_type.clone();
            }
        }
    }
    if let Some(class_info) = checker.classes.get_mut("SplDoublyLinkedList") {
        if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__debugInfo")) {
            sig.return_type = PhpType::AssocArray {
                key: Box::new(PhpType::Str),
                value: Box::new(PhpType::Mixed),
            };
        }
    }
    if let Some(class_info) = checker.classes.get_mut("IteratorIterator") {
        if let Some(sig) = class_info.methods.get_mut("__construct") {
            if let Some((_, ty)) = sig.params.first_mut() {
                *ty = PhpType::Object("Traversable".to_string());
            }
            if sig.params.len() == 1 {
                sig.params.push((
                    "class".to_string(),
                    PhpType::Union(vec![PhpType::Str, PhpType::Void]),
                ));
                sig.param_type_exprs.push(None);
                sig.defaults.push(Some(null_expr()));
                sig.ref_params.push(false);
                sig.declared_params.push(true);
            }
        }
    }
    if let Some(class_info) = checker.classes.get_mut("AppendIterator") {
        for (name, ty) in &mut class_info.properties {
            if name == "iterators" {
                *ty = PhpType::Array(Box::new(PhpType::Object("Iterator".to_string())));
            } else if name == "iteratorKeys" {
                *ty = PhpType::Array(Box::new(PhpType::Mixed));
            } else if name == "iteratorActive" {
                *ty = PhpType::Array(Box::new(PhpType::Bool));
            } else if name == "arrayIterator" {
                *ty = PhpType::Object("__ElephcAppendIteratorArrayIterator".to_string());
            }
        }
    }
    let iterator_array_type = PhpType::Array(Box::new(PhpType::Object("Iterator".to_string())));
    if let Some(class_info) = checker.classes.get_mut("MultipleIterator") {
        for (name, ty) in &mut class_info.properties {
            if name == "iterators" {
                *ty = iterator_array_type.clone();
            } else if name == "infos" {
                *ty = PhpType::Array(Box::new(PhpType::Mixed));
            }
        }
        for method in ["key", "current"] {
            if let Some(sig) = class_info.methods.get_mut(&php_symbol_key(method)) {
                sig.return_type = PhpType::Mixed;
            }
        }
    }
    if let Some(class_info) = checker.classes.get_mut("CallbackFilterIterator") {
        for (name, ty) in &mut class_info.properties {
            if name == "callback" {
                *ty = PhpType::Callable;
            } else if name == "callbackEnv" {
                *ty = PhpType::Pointer(None);
            }
        }
    }
    for class_name in [
        "RecursiveFilterIterator",
        "RecursiveCallbackFilterIterator",
        "ParentIterator",
    ] {
        if let Some(class_info) = checker.classes.get_mut(class_name) {
            for (name, ty) in &mut class_info.properties {
                if name == "callback" {
                    *ty = PhpType::Callable;
                } else if name == "callbackEnv" {
                    *ty = PhpType::Pointer(None);
                }
            }
        }
    }
    if let Some(class_info) = checker.classes.get_mut("RecursiveIteratorIterator") {
        for (name, ty) in &mut class_info.properties {
            match name.as_str() {
                "root" => *ty = PhpType::Object("RecursiveIterator".to_string()),
                "states" | "depths" => {
                    *ty = PhpType::Array(Box::new(PhpType::Int));
                }
                "iterators" => {
                    *ty = PhpType::Array(Box::new(PhpType::Object(
                        "RecursiveIterator".to_string(),
                    )));
                }
                _ => {}
            }
        }
    }
    for class_name in ["CachingIterator", "RecursiveCachingIterator"] {
        if let Some(class_info) = checker.classes.get_mut(class_name) {
            for (name, ty) in &mut class_info.properties {
                if name == "cache" {
                    *ty = PhpType::AssocArray {
                        key: Box::new(PhpType::Mixed),
                        value: Box::new(PhpType::Mixed),
                    };
                } else if name == "currentKey" || name == "currentValue" {
                    *ty = PhpType::Mixed;
                }
            }
            if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getCache")) {
                sig.return_type = PhpType::AssocArray {
                    key: Box::new(PhpType::Mixed),
                    value: Box::new(PhpType::Mixed),
                };
            }
        }
    }
}

/// Keeps inherited native list slots boxed and seeds each concrete family's iterator mode.
fn patch_dll_storage(checker: &mut Checker) {
    let classes: Vec<_> = checker.classes.keys().filter_map(|name| {
        let info = checker.classes.get(name)?;
        if !info.properties.iter().any(|(property, _)| property == DLL_STORAGE_PROPERTY) {
            return None;
        }
        let mode = if name == "SplStack" || checker.is_subclass_of(name, "SplStack") {
            6
        } else if name == "SplQueue" || checker.is_subclass_of(name, "SplQueue") {
            4
        } else {
            0
        };
        Some((name.clone(), mode))
    }).collect();
    for (name, mode) in classes {
        let info = checker.classes.get_mut(&name).expect("existing list class");
        for (index, (property, ty)) in info.properties.iter_mut().enumerate() {
            if property == DLL_STORAGE_PROPERTY {
                *ty = PhpType::Array(Box::new(PhpType::Mixed));
            } else if property == DLL_MODE_PROPERTY {
                info.defaults[index] = Some(int_expr(mode));
            }
        }
    }
}


/// Uses the PHP packed-or-hash contract on both sides of the native magic-hook ABI.
fn patch_object_storage_serialization(checker: &mut Checker) {
    for (name, info) in &mut checker.classes {
        if !info.properties.iter().any(|(property, _)| property == super::object_storage::OBJECTS_PROPERTY) {
            continue;
        }
        for method in ["__serialize", "__unserialize"] {
            let key = php_symbol_key(method);
            let declaring = info.method_declaring_classes.get(&key).map(String::as_str)
                .unwrap_or(name.as_str());
            if declaring != "SplObjectStorage" { continue; }
            let Some(sig) = info.methods.get_mut(&key) else { continue; };
            if method == "__serialize" {
                sig.return_type = PhpType::php_array();
            } else if let Some((_, ty)) = sig.params.first_mut() {
                *ty = PhpType::php_array();
            }
        }
    }
}
