//! Purpose:
//! Stores module-wide codegen artifacts that can be shared across function contexts.
//! Deduplicates runtime callable descriptors, wrappers, and invokers by semantic shape.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_module()` creates one state per generated module.
//! - `crate::codegen::lower_inst::callables` reuses emitted callable artifacts through it.
//!
//! Key details:
//! - Cached labels are global assembly entries emitted at their first call site.
//! - Receiver-bearing descriptors cache only immutable templates; each call still captures its object.
//! - Owns the module-wide assembly label counter. It must not be per function: the readable part
//!   of a label is a lossy fragment of the PHP function/block name, so only a module-unique
//!   trailing id keeps two functions with similar names from emitting the same label.
//! - Indexes emitted instance and static method bodies once from immutable EIR module metadata.

use std::collections::HashSet;

use crate::codegen::callable_dispatch::{RuntimeCallableCase, RuntimeStaticMethodCallableCase};
use crate::ir::Module;
use crate::names::php_symbol_key;
use crate::types::{FunctionSig, PhpType};

use super::shared_reflection::SharedReflectionState;

/// Module-wide artifacts emitted once and reused by every function lowering context.
pub(crate) struct SharedCodegenState {
    pub(super) reflection: SharedReflectionState,
    emitted_instance_methods: HashSet<(String, String)>,
    emitted_static_methods: HashSet<(String, String)>,
    runtime_string_descriptor_cases:
        Vec<(Option<PhpType>, Option<Vec<String>>, bool, Vec<RuntimeCallableCase>)>,
    runtime_static_method_descriptor_cases:
        Vec<(Option<Vec<String>>, Vec<RuntimeStaticMethodCallableCase>)>,
    runtime_static_method_descriptor_case_entries: Vec<RuntimeStaticMethodCallableCase>,
    runtime_instance_method_descriptors: Vec<RuntimeInstanceMethodDescriptorCacheEntry>,
    runtime_callable_invokers: Vec<RuntimeCallableInvokerCacheEntry>,
    eval_registration_helper: Option<String>,
    runtime_builtin_wrappers: Vec<RuntimeCallWrapperCacheEntry>,
    runtime_extern_wrappers: Vec<RuntimeCallWrapperCacheEntry>,
    /// Memoized sharing decision for result and stdout Mixed string contexts.
    mixed_string_sharing: [Option<bool>; 2],
    /// Memoized sharing decision for open Mixed callable dispatch by strictness profile.
    mixed_callable_sharing: [Option<bool>; 2],
    label_counter: usize,
}

/// Reusable static descriptor template for one public instance method.
#[derive(Clone)]
pub(super) struct RuntimeInstanceMethodDescriptorTemplate {
    pub(super) descriptor_label: String,
}

/// Cache key and emitted template for one receiver-class/method/signature shape.
struct RuntimeInstanceMethodDescriptorCacheEntry {
    class_name: String,
    method_key: String,
    impl_class: String,
    signature: FunctionSig,
    template: RuntimeInstanceMethodDescriptorTemplate,
}

/// Cache key and label for one signature-compatible descriptor invoker body.
struct RuntimeCallableInvokerCacheEntry {
    signature: FunctionSig,
    captures: Vec<(String, PhpType, bool)>,
    label: String,
}

/// Cache key and label for one synthetic builtin or extern entry wrapper.
struct RuntimeCallWrapperCacheEntry {
    name: String,
    signature: FunctionSig,
    strict_php: bool,
    label: String,
}

impl SharedCodegenState {
    /// Builds module-wide caches and immutable method-membership indexes before emission starts.
    pub(super) fn for_module(module: &Module) -> Self {
        let mut state = Self::empty();
        state.reflection = SharedReflectionState::for_module(module);
        for function in &module.class_methods {
            let Some((class_name, method_name)) = function.name.rsplit_once("::") else {
                continue;
            };
            let key = (class_name.to_string(), php_symbol_key(method_name));
            if function.flags.is_static {
                state.emitted_static_methods.insert(key);
            } else {
                state.emitted_instance_methods.insert(key);
            }
        }
        state
    }

    /// Creates private empty storage used only while constructing a module-populated state.
    fn empty() -> Self {
        Self {
            reflection: SharedReflectionState::empty(),
            emitted_instance_methods: HashSet::new(),
            emitted_static_methods: HashSet::new(),
            runtime_string_descriptor_cases: Vec::new(),
            runtime_static_method_descriptor_cases: Vec::new(),
            runtime_static_method_descriptor_case_entries: Vec::new(),
            runtime_instance_method_descriptors: Vec::new(),
            runtime_callable_invokers: Vec::new(),
            eval_registration_helper: None,
            runtime_builtin_wrappers: Vec::new(),
            runtime_extern_wrappers: Vec::new(),
            mixed_string_sharing: [None; 2],
            mixed_callable_sharing: [None; 2],
            label_counter: 0,
        }
    }

    /// Returns whether the immutable module inventory contains the requested method body.
    pub(super) fn emitted_method_contains(
        &self,
        class_name: &str,
        canonical_method_key: &str,
        is_static: bool,
    ) -> bool {
        let methods = if is_static {
            &self.emitted_static_methods
        } else {
            &self.emitted_instance_methods
        };
        methods.contains(&(
            class_name.to_string(),
            canonical_method_key.to_string(),
        ))
    }

    /// Borrows the immutable instance-method inventory for contains-only interface validation.
    pub(super) fn emitted_instance_method_keys(&self) -> &HashSet<(String, String)> {
        &self.emitted_instance_methods
    }

    /// Returns the memoized Mixed callable sharing decision for one strictness profile.
    pub(super) fn mixed_callable_sharing(&self, profile_index: usize) -> Option<bool> {
        self.mixed_callable_sharing[profile_index]
    }

    /// Stores the Mixed callable sharing decision for one strictness profile.
    pub(super) fn set_mixed_callable_sharing(&mut self, profile_index: usize, shares: bool) {
        self.mixed_callable_sharing[profile_index] = Some(shares);
    }

    /// Returns the memoized Mixed string sharing decision for one context mode.
    pub(super) fn mixed_string_sharing(&self, mode_index: usize) -> Option<bool> {
        self.mixed_string_sharing[mode_index]
    }

    /// Stores the Mixed string sharing decision for one context mode.
    pub(super) fn set_mixed_string_sharing(&mut self, mode_index: usize, shares: bool) {
        self.mixed_string_sharing[mode_index] = Some(shares);
    }

    /// Reserves the next module-unique assembly label id.
    ///
    /// Every generated local label ends in `_<id>` taken from this counter. Because the id is a
    /// decimal run terminated by the preceding `_`, it is recoverable from the finished label,
    /// which makes the whole label unique no matter how ambiguous its readable prefix is.
    pub(super) fn next_label_id(&mut self) -> usize {
        let id = self.label_counter;
        self.label_counter += 1;
        id
    }

    /// Returns cached runtime string-callable cases for the requested specialization.
    pub(super) fn runtime_string_descriptor_cases(
        &self,
        source_arg_ty: Option<&PhpType>,
        candidate_names: Option<&[String]>,
        strict_php: bool,
    ) -> Option<Vec<RuntimeCallableCase>> {
        self.runtime_string_descriptor_cases
            .iter()
            .find(|(cached_ty, cached_names, cached_strict_php, _)| {
                cached_ty.as_ref() == source_arg_ty
                    && cached_names.as_deref() == candidate_names
                    && *cached_strict_php == strict_php
            })
            .map(|(_, _, _, cases)| cases.clone())
    }

    /// Stores runtime string-callable cases after their global wrappers are emitted.
    pub(super) fn cache_runtime_string_descriptor_cases(
        &mut self,
        source_arg_ty: Option<&PhpType>,
        candidate_names: Option<&[String]>,
        strict_php: bool,
        cases: &[RuntimeCallableCase],
    ) {
        self.runtime_string_descriptor_cases.push((
            source_arg_ty.cloned(),
            candidate_names.map(|names| names.to_vec()),
            strict_php,
            cases.to_vec(),
        ));
    }

    /// Returns the module-wide public static-method descriptor cases, if emitted.
    pub(super) fn runtime_static_method_descriptor_cases(
        &self,
        candidate_names: Option<&[String]>,
    ) -> Option<Vec<RuntimeStaticMethodCallableCase>> {
        self.runtime_static_method_descriptor_cases
            .iter()
            .find(|(cached_names, _)| cached_names.as_deref() == candidate_names)
            .map(|(_, cases)| cases.clone())
    }

    /// Stores public static-method descriptors for reuse by later call sites.
    pub(super) fn cache_runtime_static_method_descriptor_cases(
        &mut self,
        candidate_names: Option<&[String]>,
        cases: &[RuntimeStaticMethodCallableCase],
    ) {
        self.runtime_static_method_descriptor_cases.push((
            candidate_names.map(|names| names.to_vec()),
            cases.to_vec(),
        ));
    }

    /// Returns one static-method descriptor case already emitted for another target set.
    pub(super) fn runtime_static_method_descriptor_case(
        &self,
        php_name: &str,
    ) -> Option<RuntimeStaticMethodCallableCase> {
        self.runtime_static_method_descriptor_case_entries
            .iter()
            .find(|case| case.case.php_name.as_deref() == Some(php_name))
            .cloned()
    }

    /// Records one static-method descriptor case for reuse across candidate sets.
    pub(super) fn cache_runtime_static_method_descriptor_case(
        &mut self,
        case: &RuntimeStaticMethodCallableCase,
    ) {
        self.runtime_static_method_descriptor_case_entries
            .push(case.clone());
    }

    /// Returns an emitted receiver-captured descriptor template for one method shape.
    pub(super) fn runtime_instance_method_descriptor(
        &self,
        class_name: &str,
        method_key: &str,
        impl_class: &str,
        signature: &FunctionSig,
    ) -> Option<RuntimeInstanceMethodDescriptorTemplate> {
        self.runtime_instance_method_descriptors
            .iter()
            .find(|entry| {
                entry.class_name == class_name
                    && entry.method_key == method_key
                    && entry.impl_class == impl_class
                    && entry.signature == *signature
            })
            .map(|entry| entry.template.clone())
    }

    /// Stores a receiver-captured descriptor template after first emission.
    pub(super) fn cache_runtime_instance_method_descriptor(
        &mut self,
        class_name: &str,
        method_key: &str,
        impl_class: &str,
        signature: &FunctionSig,
        template: RuntimeInstanceMethodDescriptorTemplate,
    ) {
        self.runtime_instance_method_descriptors
            .push(RuntimeInstanceMethodDescriptorCacheEntry {
                class_name: class_name.to_string(),
                method_key: method_key.to_string(),
                impl_class: impl_class.to_string(),
                signature: signature.clone(),
                template,
            });
    }

    /// Returns an already-emitted descriptor invoker with the same ABI shape.
    pub(super) fn runtime_callable_invoker(
        &self,
        signature: &FunctionSig,
        captures: &[(String, PhpType, bool)],
    ) -> Option<String> {
        self.runtime_callable_invokers
            .iter()
            .find(|entry| entry.signature == *signature && entry.captures == captures)
            .map(|entry| entry.label.clone())
    }

    /// Records a descriptor invoker body for module-wide signature reuse.
    pub(super) fn cache_runtime_callable_invoker(
        &mut self,
        signature: &FunctionSig,
        captures: &[(String, PhpType, bool)],
        label: &str,
    ) {
        self.runtime_callable_invokers
            .push(RuntimeCallableInvokerCacheEntry {
                signature: signature.clone(),
                captures: captures.to_vec(),
                label: label.to_string(),
            });
    }

    /// Returns the module-wide eval metadata registration helper label, if emitted.
    pub(super) fn eval_registration_helper(&self) -> Option<String> {
        self.eval_registration_helper.clone()
    }

    /// Publishes the module-wide eval metadata registration helper label.
    pub(super) fn cache_eval_registration_helper(&mut self, label: String) {
        debug_assert!(self.eval_registration_helper.is_none());
        self.eval_registration_helper = Some(label);
    }

    /// Returns a previously emitted synthetic builtin wrapper for the same signature.
    pub(super) fn runtime_builtin_wrapper(
        &self,
        name: &str,
        signature: &FunctionSig,
        strict_php: bool,
    ) -> Option<String> {
        cached_runtime_call_wrapper(
            &self.runtime_builtin_wrappers,
            name,
            signature,
            strict_php,
        )
    }

    /// Records a synthetic builtin wrapper for module-wide reuse.
    pub(super) fn cache_runtime_builtin_wrapper(
        &mut self,
        name: &str,
        signature: &FunctionSig,
        strict_php: bool,
        label: &str,
    ) {
        cache_runtime_call_wrapper(
            &mut self.runtime_builtin_wrappers,
            name,
            signature,
            strict_php,
            label,
        );
    }

    /// Returns a previously emitted synthetic extern wrapper for the same signature.
    pub(super) fn runtime_extern_wrapper(
        &self,
        name: &str,
        signature: &FunctionSig,
    ) -> Option<String> {
        cached_runtime_call_wrapper(&self.runtime_extern_wrappers, name, signature, false)
    }

    /// Records a synthetic extern wrapper for module-wide reuse.
    pub(super) fn cache_runtime_extern_wrapper(
        &mut self,
        name: &str,
        signature: &FunctionSig,
        label: &str,
    ) {
        cache_runtime_call_wrapper(
            &mut self.runtime_extern_wrappers,
            name,
            signature,
            false,
            label,
        );
    }
}

/// Looks up a cached synthetic call wrapper by PHP name and ABI signature.
fn cached_runtime_call_wrapper(
    entries: &[RuntimeCallWrapperCacheEntry],
    name: &str,
    signature: &FunctionSig,
    strict_php: bool,
) -> Option<String> {
    entries
        .iter()
        .find(|entry| {
            entry.name == name
                && entry.signature == *signature
                && entry.strict_php == strict_php
        })
        .map(|entry| entry.label.clone())
}

/// Adds one synthetic call wrapper to its module-wide cache.
fn cache_runtime_call_wrapper(
    entries: &mut Vec<RuntimeCallWrapperCacheEntry>,
    name: &str,
    signature: &FunctionSig,
    strict_php: bool,
    label: &str,
) {
    entries.push(RuntimeCallWrapperCacheEntry {
        name: name.to_string(),
        signature: signature.clone(),
        strict_php,
        label: label.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::Target;
    use crate::ir::{Function, IrType};

    /// Creates a minimal EIR method body with the requested raw name and static flag.
    fn method(name: &str, is_static: bool) -> Function {
        let mut function = Function::new(name.to_string(), IrType::Void, PhpType::Void);
        function.flags.is_static = is_static;
        function
    }

    /// Reproduces the pre-index linear membership predicate for differential assertions.
    fn old_scan_contains(
        module: &Module,
        class_name: &str,
        method_key: &str,
        is_static: bool,
    ) -> bool {
        module.class_methods.iter().any(|function| {
            function.flags.is_static == is_static
                && function
                    .name
                    .rsplit_once("::")
                    .is_some_and(|(candidate_class, candidate_method)| {
                        candidate_class == class_name
                            && php_symbol_key(candidate_method) == method_key
                    })
        })
    }

    /// Proves the immutable index preserves raw classes and method-only canonicalization.
    #[test]
    fn emitted_method_index_matches_previous_scan_semantics() {
        let mut module = Module::new(Target::detect_host());
        module.class_methods = vec![
            method("Ns\\Thing::DoWork", false),
            method("Ns\\Thing::StaticWork", true),
            method("Ns\\Thing::Dual", false),
            method("Ns\\Thing::Dual", true),
            method("Outer::Inner::MiXeD", false),
            method("\\Ns\\Thing::Leading", false),
            method("missing_delimiter", false),
        ];
        let state = SharedCodegenState::for_module(&module);
        let queries = [
            ("Ns\\Thing", "dowork", false),
            ("Ns\\Thing", "DOWORK", false),
            ("Ns\\Thing", "staticwork", true),
            ("Ns\\Thing", "staticwork", false),
            ("Ns\\Thing", "dual", false),
            ("Ns\\Thing", "dual", true),
            ("Outer::Inner", "mixed", false),
            ("Outer", "inner::mixed", false),
            ("\\Ns\\Thing", "leading", false),
            ("Ns\\Thing", "leading", false),
            ("", "missing_delimiter", false),
        ];

        for (class_name, method_key, is_static) in queries {
            assert_eq!(
                state.emitted_method_contains(class_name, method_key, is_static),
                old_scan_contains(&module, class_name, method_key, is_static),
                "membership mismatch for {class_name}::{method_key}, static={is_static}"
            );
        }
    }
}
