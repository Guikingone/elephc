//! Purpose:
//! Class hierarchy, interface, trait, and class capability metadata.
//!
//! Called from:
//! - `crate::codegen::lower_inst::objects::reflection`.
//!
//! Key details:
//! - Preserves compile-time metadata, target-aware object layout, and ownership.

use super::*;

/// Returns the canonical parent class name for a reflected class, if any.
pub(super) fn reflection_parent_class_name(
    ctx: &FunctionContext<'_>,
    info: &crate::types::ClassInfo,
) -> Option<String> {
    let parent = info.parent.as_ref()?;
    resolve_reflection_class(ctx, parent)
        .map(|(parent_name, _)| parent_name.to_string())
        .or_else(|| Some(parent.trim_start_matches('\\').to_string()))
}

/// Returns direct and inherited parent class names for `ReflectionClass::isSubclassOf()`.
pub(super) fn reflection_parent_class_names(
    ctx: &FunctionContext<'_>,
    info: &crate::types::ClassInfo,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut current = reflection_parent_class_name(ctx, info);
    while let Some(parent_name) = current {
        if names
            .iter()
            .any(|name| php_symbol_key(name) == php_symbol_key(&parent_name))
        {
            break;
        }
        current = resolve_reflection_class(ctx, &parent_name)
            .and_then(|(_, parent_info)| reflection_parent_class_name(ctx, parent_info));
        names.push(parent_name);
    }
    names
}

/// Returns PHP's `ReflectionClass::isInstantiable()` value for static class metadata.
pub(super) fn reflection_class_is_instantiable(
    info: &crate::types::ClassInfo,
    is_enum: bool,
    constructor_member: Option<&ReflectionListedMember>,
) -> bool {
    if info.is_abstract || is_enum {
        return false;
    }
    constructor_member
        .map(|member| member.flags.is_public)
        .unwrap_or(true)
}

/// Returns PHP/elephc cloneability for a reflected class.
pub(super) fn reflection_class_is_cloneable(
    class_name: &str,
    info: &crate::types::ClassInfo,
    is_enum: bool,
) -> bool {
    if info.is_abstract || is_enum || reflection_class_has_runtime_managed_storage(class_name) {
        return false;
    }
    let clone_key = php_symbol_key("__clone");
    info.method_visibilities
        .get(&clone_key)
        .is_none_or(|visibility| matches!(visibility, Visibility::Public))
}

/// Returns PHP's `ReflectionClass::isIterable()` value for static class metadata.
pub(super) fn reflection_class_is_iterable(info: &crate::types::ClassInfo, is_enum: bool) -> bool {
    if info.is_abstract || is_enum {
        return false;
    }
    info.interfaces
        .iter()
        .any(|name| name == "Iterator" || name == "IteratorAggregate")
}

/// Returns whether a builtin's object layout is outside ordinary declared slots.
pub(super) fn reflection_class_has_runtime_managed_storage(class_name: &str) -> bool {
    let key = php_symbol_key(class_name);
    matches!(
        key.as_str(),
        "throwable"
            | "error"
            | "exception"
            | "valueerror"
            | "runtimeexception"
            | "reflectionexception"
            | "jsonexception"
            | "fiber"
            | "fibererror"
            | "generator"
            | "reflectionattribute"
            | "reflectionclass"
            | "reflectionfunction"
            | "reflectionmethod"
            | "reflectionproperty"
            | "reflectionparameter"
            | "reflectionnamedtype"
            | "reflectionuniontype"
            | "reflectionintersectiontype"
            | "reflectionclassconstant"
            | "reflectionenumunitcase"
            | "reflectionenumbackedcase"
            | "splfixedarray"
            | "spldoublylinkedlist"
            | "splstack"
            | "splqueue"
            | "iteratoriterator"
            | "filteriterator"
            | "callbackfilteriterator"
            | "recursivefilteriterator"
            | "recursivecallbackfilteriterator"
            | "recursiveiteratoriterator"
    )
}

/// Returns whether the reflected class-like name belongs to compiler-injected PHP metadata.
pub(super) fn reflection_class_like_is_internal(class_name: &str) -> bool {
    let key = php_symbol_key(class_name.trim_start_matches('\\'));
    matches!(
        key.as_str(),
        "__elephcappenditeratorarrayiterator"
            | "appenditerator"
            | "arrayaccess"
            | "arrayiterator"
            | "arrayobject"
            | "badfunctioncallexception"
            | "badmethodcallexception"
            | "cachingiterator"
            | "callbackfilteriterator"
            | "countable"
            | "directoryiterator"
            | "domainexception"
            | "emptyiterator"
            | "error"
            | "exception"
            | "fiber"
            | "fibererror"
            | "filteriterator"
            | "filesystemiterator"
            | "generator"
            | "globiterator"
            | "infiniteiterator"
            | "internaliterator"
            | "invalidargumentexception"
            | "iterator"
            | "iteratoraggregate"
            | "iteratoriterator"
            | "jsonexception"
            | "jsonserializable"
            | "lengthexception"
            | "limititerator"
            | "logicexception"
            | "multipleiterator"
            | "norewinditerator"
            | "outeriterator"
            | "outofboundsexception"
            | "outofrangeexception"
            | "overflowexception"
            | "parentiterator"
            | "phar"
            | "phardata"
            | "pharfileinfo"
            | "php_user_filter"
            | "rangeexception"
            | "recursivearrayiterator"
            | "recursivecachingiterator"
            | "recursivecallbackfilteriterator"
            | "recursivedirectoryiterator"
            | "recursivefilteriterator"
            | "recursiveiterator"
            | "recursiveiteratoriterator"
            | "recursiveregexiterator"
            | "reflectionattribute"
            | "reflectionclass"
            | "reflectionclassconstant"
            | "reflectionenumbackedcase"
            | "reflectionenumunitcase"
            | "reflectionexception"
            | "reflectionfunction"
            | "reflectionintersectiontype"
            | "reflectionmethod"
            | "reflectionnamedtype"
            | "reflectionparameter"
            | "reflectionproperty"
            | "reflectionuniontype"
            | "regexiterator"
            | "runtimeexception"
            | "seekableiterator"
            | "sortdirection"
            | "spldoublylinkedlist"
            | "splfixedarray"
            | "splfileinfo"
            | "splfileobject"
            | "splheap"
            | "splmaxheap"
            | "splminheap"
            | "splobjectstorage"
            | "splobserver"
            | "splpriorityqueue"
            | "splqueue"
            | "splstack"
            | "splsubject"
            | "spltempfileobject"
            | "stdclass"
            | "stringable"
            | "throwable"
            | "traversable"
            | "typeerror"
            | "underflowexception"
            | "unexpectedvalueexception"
            | "valueerror"
    )
}

/// Collects direct and inherited parent interfaces for a reflected interface.
pub(super) fn reflection_interface_parent_names(
    ctx: &FunctionContext<'_>,
    interface_name: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    collect_reflection_interface_parent_names(ctx, interface_name, &mut names);
    names
}

/// Recursively collects interface parents without duplicating case-insensitive names.
pub(super) fn collect_reflection_interface_parent_names(
    ctx: &FunctionContext<'_>,
    interface_name: &str,
    names: &mut Vec<String>,
) {
    let Some(interface) = ctx.module.interface_infos.get(interface_name) else {
        return;
    };
    for parent in &interface.parents {
        let parent_name = resolve_reflection_interface(ctx, parent)
            .map(str::to_string)
            .unwrap_or_else(|| parent.clone());
        if !names
            .iter()
            .any(|name| php_symbol_key(name) == php_symbol_key(&parent_name))
        {
            names.push(parent_name.clone());
            collect_reflection_interface_parent_names(ctx, &parent_name, names);
        }
    }
}

