//! Purpose:
//! Patches ReflectionProperty, ReflectionMethod, and ReflectionFunction signatures.
//!
//! Called from:
//! - patch_builtin_reflection_signatures() through the Reflection checker facade.
//!
//! Key details:
//! - Invocation, hook, modifier, and parameter collection contracts remain owner-specific.

use super::*;

/// Applies ReflectionProperty-specific signature overrides.
pub(super) fn patch_reflection_property(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionProperty" {
                for (property_name, property_type) in &mut class_info.properties {
                    if property_name == "__hooks" {
                        *property_type = reflection_property_hook_map_type();
                    }
                }
                for method_name in [
                    "isfinal",
                    "isabstract",
                    "isreadonly",
                    "isdefault",
                    "ispromoted",
                    "isvirtual",
                    "isdynamic",
                    "hashooks",
                    "isinitialized",
                    "isprotectedset",
                    "isprivateset",
                ] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                if let Some(sig) = class_info
                    .methods
                    .get_mut(&php_symbol_key("hasDefaultValue"))
                {
                    sig.return_type = PhpType::Bool;
                }
                if let Some(sig) = class_info
                    .methods
                    .get_mut(&php_symbol_key("getDefaultValue"))
                {
                    sig.return_type = PhpType::Mixed;
                }
                for method_name in ["getType", "getSettableType"] {
                    if let Some(sig) = class_info.methods.get_mut(&php_symbol_key(method_name)) {
                        sig.return_type = PhpType::Mixed;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getHooks")) {
                    sig.return_type = reflection_property_hook_map_type();
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("hasHook")) {
                    sig.params = vec![(
                        "type".to_string(),
                        PhpType::Object("PropertyHookType".to_string()),
                    )];
                    sig.param_type_exprs =
                        vec![Some(TypeExpr::Named(Name::unqualified("PropertyHookType")))];
                    sig.defaults = vec![None];
                    sig.ref_params = vec![false];
                    sig.declared_params = vec![true];
                    sig.return_type = PhpType::Bool;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getHook")) {
                    sig.params = vec![(
                        "type".to_string(),
                        PhpType::Object("PropertyHookType".to_string()),
                    )];
                    sig.param_type_exprs =
                        vec![Some(TypeExpr::Named(Name::unqualified("PropertyHookType")))];
                    sig.defaults = vec![None];
                    sig.ref_params = vec![false];
                    sig.declared_params = vec![true];
                    sig.return_type = PhpType::Union(vec![
                        PhpType::Object("ReflectionMethod".to_string()),
                        PhpType::Void,
                    ]);
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getModifiers")) {
                    sig.return_type = PhpType::Int;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
            }
}

/// Applies ReflectionMethod-specific signature overrides.
pub(super) fn patch_reflection_method(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionMethod" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
                for method_name in [
                    "isfinal",
                    "isabstract",
                    "isconstructor",
                    "isdestructor",
                    "hasPrototype",
                ] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getPrototype")) {
                    sig.return_type = PhpType::Object("ReflectionMethod".to_string());
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getModifiers")) {
                    sig.return_type = PhpType::Int;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("invoke")) {
                    sig.return_type = PhpType::Mixed;
                    sig.variadic = Some("args".to_string());
                    make_reflection_variadic_optional(sig);
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("invokeArgs")) {
                    // Keep the shell's (object, args) parameters: ReflectionMethod::invokeArgs
                    // takes the receiver first, and replacing the params with a
                    // lone args array breaks invoke dispatch.
                    sig.return_type = PhpType::Mixed;
                }
                if let Some(sig) =
                    class_info.methods.get_mut(&php_symbol_key("createFromMethodName"))
                {
                    sig.params = vec![("method".to_string(), PhpType::Str)];
                    sig.param_type_exprs = vec![Some(TypeExpr::Str)];
                    sig.defaults = vec![None];
                    sig.ref_params = vec![false];
                    sig.declared_params = vec![true];
                    sig.return_type = PhpType::Object("ReflectionMethod".to_string());
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getParameters")) {
                    sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                        "ReflectionParameter".to_string(),
                    )));
                }
                for method_name in ["getNumberOfParameters", "getNumberOfRequiredParameters"] {
                    if let Some(sig) = class_info.methods.get_mut(&php_symbol_key(method_name)) {
                        sig.return_type = PhpType::Int;
                    }
                }
            }
}

/// Applies ReflectionFunction-specific signature overrides.
pub(super) fn patch_reflection_function(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionFunction" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("invoke")) {
                    sig.return_type = PhpType::Mixed;
                    sig.variadic = Some("args".to_string());
                    make_reflection_variadic_optional(sig);
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("invokeArgs")) {
                    sig.return_type = PhpType::Mixed;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getParameters")) {
                    sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                        "ReflectionParameter".to_string(),
                    )));
                }
                for method_name in ["getNumberOfParameters", "getNumberOfRequiredParameters"] {
                    if let Some(sig) = class_info.methods.get_mut(&php_symbol_key(method_name)) {
                        sig.return_type = PhpType::Int;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("isDisabled")) {
                    sig.return_type = PhpType::Bool;
                }
            }
}
