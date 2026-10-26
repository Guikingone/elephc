//! Purpose:
//! Emits PHP method adapters for runtime-backed intrinsic implementations.
//!
//! Called from:
//! - `crate::codegen::finalize_user_asm()` and runtime class method discovery.
//!
//! Key details:
//! - Source method arguments are borrowed; consuming helpers receive independent owners.
//! - The intrinsic catalog supplies consumed positions and ABI helpers preserve all targets.

use super::*;
use crate::codegen_support::abi;

/// Emits method-symbol wrappers for runtime-backed intrinsic class methods.
pub(in crate::codegen) fn emit_intrinsic_method_wrappers(module: &Module, emitter: &mut Emitter) {
    for wrapper in intrinsic_method_wrapper_specs(module) {
        let symbol = if wrapper.is_static {
            static_method_symbol(&wrapper.class_name, &wrapper.method_key)
        } else {
            method_symbol(&wrapper.class_name, &wrapper.method_key)
        };
        emitter.label(&symbol);
        emit_consumed_parameter_owners(emitter, &wrapper);
        match emitter.target.arch {
            Arch::AArch64 => {
                emitter.instruction(&format!("b {}", wrapper.helper));          // tail-call the runtime helper using the method ABI arguments
            }
            Arch::X86_64 => {
                emitter.instruction(&format!("jmp {}", wrapper.helper));        // tail-call the runtime helper using the method ABI arguments
            }
        }
    }
}

/// Runtime-backed method wrapper that should be emitted as a PHP method symbol.
pub(in crate::codegen) struct IntrinsicMethodWrapper {
    pub(in crate::codegen) class_name: String,
    pub(in crate::codegen) method_key: String,
    helper: &'static str,
    pub(in crate::codegen) is_static: bool,
    consumed_mixed_parameters: &'static [usize],
    integer_argument_count: usize,
}

/// Returns intrinsic instance/static methods that need method-symbol wrappers.
pub(in crate::codegen) fn intrinsic_method_wrapper_specs(module: &Module) -> Vec<IntrinsicMethodWrapper> {
    let eir_methods = eir_class_method_keys(module);
    let mut wrappers = Vec::new();
    for (class_name, class_info) in &module.class_infos {
        for method_key in class_info.methods.keys() {
            let impl_class = class_info
                .method_impl_classes
                .get(method_key)
                .map(String::as_str)
                .unwrap_or(class_name.as_str());
            if eir_methods.contains(&(impl_class.to_string(), method_key.clone(), false)) {
                continue;
            }
            if let Some((intrinsic, helper)) = IntrinsicCall::instance_method(impl_class, method_key)
                .and_then(|intrinsic| intrinsic.runtime_helper().map(|helper| (intrinsic, helper)))
            {
                let signature = &class_info.methods[method_key];
                for &index in intrinsic.consumed_mixed_parameters() {
                    assert_eq!(signature.params[index].1.codegen_repr(), PhpType::Mixed);
                }
                wrappers.push(IntrinsicMethodWrapper {
                    class_name: impl_class.to_string(),
                    method_key: method_key.clone(),
                    helper,
                    is_static: false,
                    consumed_mixed_parameters: intrinsic.consumed_mixed_parameters(),
                    integer_argument_count: signature.params.len() + 1,
                });
            }
        }
        for method_key in class_info.static_methods.keys() {
            let impl_class = class_info
                .static_method_impl_classes
                .get(method_key)
                .map(String::as_str)
                .unwrap_or(class_name.as_str());
            if eir_methods.contains(&(impl_class.to_string(), method_key.clone(), true)) {
                continue;
            }
            if let Some(helper) = IntrinsicCall::static_method(impl_class, method_key)
                .and_then(|intrinsic| intrinsic.runtime_helper())
            {
                wrappers.push(IntrinsicMethodWrapper {
                    class_name: impl_class.to_string(),
                    method_key: method_key.clone(),
                    helper,
                    is_static: true,
                    consumed_mixed_parameters: &[],
                    integer_argument_count: 0,
                });
            }
        }
    }
    wrappers.sort_by(|left, right| {
        (&left.class_name, &left.method_key, left.is_static).cmp(&(
            &right.class_name,
            &right.method_key,
            right.is_static,
        ))
    });
    wrappers.dedup_by(|left, right| {
        left.class_name == right.class_name
            && left.method_key == right.method_key
            && left.is_static == right.is_static
    });
    wrappers
}

/// Acquires consuming helper arguments without moving the PHP caller's borrowed references.
fn emit_consumed_parameter_owners(emitter: &mut Emitter, wrapper: &IntrinsicMethodWrapper) {
    if wrapper.consumed_mixed_parameters.is_empty() {
        return;
    }
    // The reviewed SPL consuming intrinsics carry only a receiver and at most two words.
    assert!(wrapper.integer_argument_count <= 3);
    const FRAME_SIZE: usize = 48;
    abi::emit_frame_prologue(emitter, FRAME_SIZE);
    for index in 0..wrapper.integer_argument_count {
        let register = abi::int_arg_reg_name(emitter.target, index);
        abi::store_at_offset(emitter, register, (index + 1) * 8);
    }
    for &parameter in wrapper.consumed_mixed_parameters {
        let result = abi::int_result_reg(emitter);
        abi::load_at_offset(emitter, result, (parameter + 2) * 8);
        abi::emit_call_label(emitter, "__rt_incref");
    }
    for index in 0..wrapper.integer_argument_count {
        let register = abi::int_arg_reg_name(emitter.target, index);
        abi::load_at_offset(emitter, register, (index + 1) * 8);
    }
    abi::emit_frame_restore(emitter, FRAME_SIZE);
}
