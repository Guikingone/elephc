//! Purpose:
//! Resolves concrete date methods callable through a DateTimeInterface receiver.
//!
//! Called from:
//! - Method and callable checking and AST-to-EIR method signature resolution.
//!
//! Key details:
//! - PHP prohibits user implementations of DateTimeInterface; receivers belong to
//!   the DateTime or DateTimeImmutable families. Reflection stays declaration-based.

use std::collections::HashMap;

use super::{ClassInfo, FunctionSig, PhpType};

/// Resolves a public method shared by both concrete date families without adding
/// a fictitious declaration to DateTimeInterface's reflected method table.
pub(crate) fn concrete_date_interface_method(
    classes: &HashMap<String, ClassInfo>,
    receiver: &str,
    method: &str,
) -> Option<FunctionSig> {
    if !receiver.trim_start_matches('\\').eq_ignore_ascii_case("DateTimeInterface") {
        return None;
    }
    let key = crate::names::php_symbol_key(method);
    let mut signatures = ["DateTime", "DateTimeImmutable"].into_iter().map(|class| {
        if super::date_reflection_signatures::php_src_date_method_visible(class, &key)
            != Some(true)
        {
            return None;
        }
        let mut signature = classes.get(class)?.methods.get(&key)?.clone();
        signature.return_type = date_family_result(signature.return_type);
        Some(signature)
    });
    let signature = signatures.next()??;
    let other = signatures.next()??;
    if signature.return_type != other.return_type {
        return None;
    }
    Some(signature)
}

/// Preserves raw object storage while hiding which date family a runtime method returns.
fn date_family_result(ty: PhpType) -> PhpType {
    match ty {
        PhpType::Object(ref class) if matches!(class.as_str(), "DateTime" | "DateTimeImmutable") => {
            PhpType::Object("DateTimeInterface".into())
        }
        PhpType::Union(members) => {
            PhpType::Union(members.into_iter().map(date_family_result).collect())
        }
        other => other,
    }
}
