//! Purpose:
//! Patches ReflectionParameter and reflected type-object signatures.
//!
//! Called from:
//! - patch_builtin_reflection_signatures() through the Reflection checker facade.
//!
//! Key details:
//! - Nullable type objects and composite member arrays retain their PHP-facing shapes.

use super::*;

/// Applies ReflectionParameter-specific signature overrides.
pub(super) fn patch_reflection_parameter(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionParameter" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getPosition")) {
                    sig.return_type = PhpType::Int;
                }
                for method_name in [
                    "isoptional",
                    "isvariadic",
                    "ispassedbyreference",
                    "canbepassedbyvalue",
                    "ispromoted",
                    "hastype",
                    "allowsnull",
                    "isarray",
                    "iscallable",
                    "isdefaultvalueavailable",
                    "isdefaultvalueconstant",
                ] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                for method_name in ["getType", "getDefaultValue", "getDefaultValueConstantName"] {
                    if let Some(sig) = class_info.methods.get_mut(&php_symbol_key(method_name)) {
                        sig.return_type = PhpType::Mixed;
                    }
                }
            }
}

/// Applies ReflectionNamedType-specific signature overrides.
pub(super) fn patch_reflection_named_type(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionNamedType" {
                for method_name in ["allowsnull", "isbuiltin"] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
            }
}

/// Applies ReflectionUnionType-specific signature overrides.
pub(super) fn patch_reflection_union_type(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionUnionType" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getTypes")) {
                    sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                        "ReflectionNamedType".to_string(),
                    )));
                }
                for method_name in ["allowsnull", "isbuiltin"] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
            }
}

/// Applies ReflectionIntersectionType-specific signature overrides.
pub(super) fn patch_reflection_intersection_type(class_name: &str, class_info: &mut ClassInfo) {
            if class_name == "ReflectionIntersectionType" {
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("getTypes")) {
                    sig.return_type = PhpType::Array(Box::new(PhpType::Object(
                        "ReflectionNamedType".to_string(),
                    )));
                }
                for method_name in ["allowsnull", "isbuiltin"] {
                    if let Some(sig) = class_info.methods.get_mut(method_name) {
                        sig.return_type = PhpType::Bool;
                    }
                }
                if let Some(sig) = class_info.methods.get_mut(&php_symbol_key("__toString")) {
                    sig.return_type = PhpType::Str;
                }
            }
}
