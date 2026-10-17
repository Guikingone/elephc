//! Purpose:
//! Shares PHP Core attribute target masks between native and interpreted validation.
//!
//! Called from:
//! - Compiler declaration checks, reflection factories, and Magician attribute construction.
//!
//! Key details:
//! - Bit 64 means repeatable; the low six bits are Attribute::TARGET_* flags.
//! - Namespaced user attributes never match a global predefined attribute.

/// Returns predefined target and repeatability flags for a canonical global attribute name.
pub fn core_attribute_flags(name: &str) -> Option<i64> {
    Some(match name.trim_start_matches('\\').to_ascii_lowercase().as_str() {
        "attribute" | "allowdynamicproperties" => 1,
        "returntypewillchange" => 4,
        "sensitiveparameter" => 32,
        "override" => 12,
        "deprecated" => 87,
        "nodiscard" => 6,
        "delayedtargetvalidation" => 127,
        _ => return None,
    })
}
