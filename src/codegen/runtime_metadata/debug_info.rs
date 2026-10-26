//! Purpose:
//! Emits native magic-debug descriptors with the ordinary callable return ABI.
//!
//! Called from:
//! - `crate::codegen::finalize_user_asm()` after source method adapters are available.
//!
//! Key details:
//! - Templates describe one receiver capture and no PHP-visible arguments.
//! - Callers must bind and root that receiver before invoking the descriptor.
//! - Existing callable invokers normalize every return representation to an owned Mixed cell.

use std::collections::{HashMap, HashSet};

use crate::codegen::runtime_callable_invoker::{
    emit_runtime_callable_invoker, method_returns_owned_string, RuntimeCallableInvoker,
};
use crate::codegen::{CodegenIrError, Result};
use crate::codegen_support::abi;
use crate::codegen_support::callable_descriptor::{self, CallableDescriptorInvocation, CallableDescriptorShape};
use crate::codegen_support::data_section::DataSection;
use crate::codegen_support::emit::Emitter;
use crate::ir::Module;
use crate::names::method_symbol;
use crate::types::{ClassInfo, PhpType};

/// Emits boxed-return invokers and a dense table of immutable debug-method templates.
pub(in crate::codegen) fn emit_debug_info_metadata(
    module: &Module,
    classes: &HashMap<String, ClassInfo>,
    allowed: &HashSet<String>,
    emitter: &mut Emitter,
    data: &mut DataSection,
) -> Result<String> {
    let mut classes = classes.iter()
        .filter(|(name, _)| allowed.contains(*name))
        .collect::<Vec<_>>();
    classes.sort_by_key(|(_, info)| info.class_id);
    let count = classes.last().map_or(0, |(_, info)| info.class_id + 1);
    let mut templates = HashMap::new();
    for (name, info) in classes {
        let Some(implementation) = info.method_impl_classes.get("__debuginfo") else {
            continue;
        };
        let Some(signature) = info.methods.get("__debuginfo") else {
            continue;
        };
        let visible_regular = crate::types::call_args::regular_param_count(signature);
        let visible_variadic = signature.variadic.as_deref()
            .is_some_and(|name| name != crate::func_args::HIDDEN_ARGS_PARAM);
        if visible_regular != 0 || visible_variadic {
            return Err(CodegenIrError::unsupported(format!(
                "{name}::__debugInfo() with PHP-visible parameters {:?}", signature.params,
            )));
        }
        // The ordinary invoker synthesizes hidden func_args slots from the empty public
        // container. Keep the physical signature so that this ABI work is not discarded.
        let entry = emit_debug_info_entry(
            emitter, info.class_id, implementation, signature.params.len(),
        );
        let captures = [("receiver".to_string(), PhpType::Object(name.clone()), false)];
        let invoker_label = format!("_object_debug_info_invoker_{}", info.class_id);
        emit_runtime_callable_invoker(emitter, data, &RuntimeCallableInvoker {
            label: &invoker_label,
            sig: signature,
            captures: &captures,
            mbstring_operation: None,
            mbstring_variable_ref_warnings: false,
            owns_string_return: method_returns_owned_string(
                module, implementation, "__debuginfo", false,
            ),
            php_return_status: true,
            defaults: &[],
        });
        let php_name = format!("{implementation}::__debugInfo");
        let template = callable_descriptor::static_descriptor_with_optional_invoker_meta(
            data,
            &entry,
            Some(&php_name),
            callable_descriptor::CALLABLE_DESC_KIND_FIRST_CLASS,
            Some(signature),
            &captures,
            &[],
            CallableDescriptorInvocation::method(
                CallableDescriptorShape::InstanceMethod,
                Some(name.clone()),
                "__debugInfo",
            ),
            Some(&invoker_label),
        );
        templates.insert(info.class_id, template);
    }
    let mut out = format!(
        "\n.p2align 3\n.globl _object_debug_info_template_count\n_object_debug_info_template_count:\n    .quad {count}\n.globl _object_debug_info_templates\n_object_debug_info_templates:\n",
    );
    for id in 0..count {
        out.push_str(&format!("    .quad {}\n", templates.get(&id).map_or("0", String::as_str)));
    }
    out.push_str(concat!(
        ".p2align 3\n.globl _object_debug_info_empty_args\n_object_debug_info_empty_args:\n",
        "    .quad 4\n    .quad _object_debug_info_empty_array\n    .quad 0\n",
        "_object_debug_info_empty_array:\n    .quad 0\n    .quad 7\n    .quad 0\n",
    ));
    Ok(out)
}

/// Reorders synthesized physical arguments ahead of the captured receiver for method entry.
fn emit_debug_info_entry(
    emitter: &mut Emitter,
    class_id: u64,
    implementation: &str,
    hidden_count: usize,
) -> String {
    let method = method_symbol(implementation, "__debuginfo");
    if hidden_count == 0 {
        return method;
    }
    // The shared invoker appends captures after its hidden argument collector. Methods
    // receive `this` first. A tail adapter preserves the invoker's ownership and return ABI.
    let entry = format!("_object_debug_info_entry_{class_id}");
    emitter.label_global(&entry);
    let scratch = abi::symbol_scratch_reg(emitter);
    let receiver = abi::int_arg_reg_name(emitter.target, hidden_count);
    abi::emit_reg_move(emitter, scratch, receiver);
    for index in (0..hidden_count).rev() {
        let source = abi::int_arg_reg_name(emitter.target, index);
        let destination = abi::int_arg_reg_name(emitter.target, index + 1);
        abi::emit_reg_move(emitter, destination, source);
    }
    let first = abi::int_arg_reg_name(emitter.target, 0);
    abi::emit_reg_move(emitter, first, scratch);
    abi::emit_jump(emitter, &method);
    entry
}
