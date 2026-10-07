//! Purpose:
//! Applies generic PHPDoc to properties and methods, including method-local templates.
//! Keeps class scope separate from templates bound by an individual method call.
//!
//! Called from:
//! - `crate::docblock::apply_to_stmt()` for class, interface and trait members.
//!
//! Key details:
//! - Native method templates take precedence over PHPDoc.
//! - Ordinary member annotations only apply when they mention a class type parameter.
//! - An explicit method template applies its parameter and return annotations like a function.
//! - Promoted properties retain the annotated constructor parameter's storage type.

use std::collections::HashMap;

use crate::parser::ast::{ClassMethod, ClassProperty, TypeExpr};

use super::DocBlock;

/// Applies class-scoped member annotations and adopts each method's own template declaration.
///
/// `owner_line` is the line of the class-like declaration itself. A member that starts on that
/// line (`class Box { public function __construct(public T $v) {} }`) has no doc comment of its
/// own: the block keyed to that line belongs to the declaration, and copying its `@template T`
/// onto the constructor made `new Box<int>(7)` expect no arguments.
pub(super) fn apply(
    class_params: &[String],
    properties: &mut [ClassProperty],
    methods: &mut [ClassMethod],
    blocks: &HashMap<usize, DocBlock>,
    owner_line: usize,
) {
    let member_block = |line: u32| {
        let line = line as usize;
        (line != owner_line).then(|| blocks.get(&line)).flatten()
    };
    for property in properties.iter_mut() {
        if let Some(annotated) = member_block(property.span.line)
            .and_then(|block| block.var_type.as_ref())
        {
            apply_type(&mut property.type_expr, annotated, class_params, false);
        }
    }
    let mut promoted = Vec::new();
    for method in methods.iter_mut() {
        let Some(block) = member_block(method.span.line) else {
            continue;
        };
        if !method.type_params.is_empty() {
            continue;
        }
        let is_constructor = method.name.eq_ignore_ascii_case("__construct");
        // A constructor's `@template` is not adopted: its parameter would type the object being
        // built, which only a CLASS template can monomorphize, and the native form is refused for
        // that reason. php ignores the annotation, so ignoring it keeps the file compiling as php
        // reads it; adopting it erased the constructor and `new Box(5)` expected no arguments.
        let is_template = block.declares_generics() && !is_constructor;
        if is_template {
            method.type_params = block.type_params.clone();
        }
        for (name, declared, _, _) in method.params.iter_mut() {
            let Some(annotated) = block.params.get(name) else {
                continue;
            };
            if apply_type(declared, annotated, class_params, is_template) && is_constructor {
                promoted.push((name.clone(), annotated.clone()));
            }
        }
        if let Some(annotated) = method
            .variadic
            .as_ref()
            .and_then(|name| block.params.get(name))
        {
            apply_type(&mut method.variadic_type, annotated, class_params, is_template);
        }
        if let Some(annotated) = &block.return_type {
            apply_type(&mut method.return_type, annotated, class_params, is_template);
        }
    }
    // The parser creates a promoted parameter and its property separately. Both must agree.
    for property in properties.iter_mut().filter(|property| property.is_promoted) {
        if let Some((_, annotated)) = promoted.iter().find(|(name, _)| *name == property.name) {
            property.type_expr = Some(annotated.clone());
        }
    }
}

/// Applies template annotations, retaining the class-parameter gate for ordinary members.
fn apply_type(
    declared: &mut Option<TypeExpr>,
    annotated: &TypeExpr,
    class_params: &[String],
    method_template: bool,
) -> bool {
    if !method_template && !annotated.mentions_type_param(class_params) {
        return false;
    }
    *declared = Some(annotated.clone());
    true
}
