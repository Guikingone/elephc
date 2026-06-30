//! Purpose:
//! Emits the closed-world constant and enum registry data tables consumed by the
//! `__rt_defined`/`__rt_constant`/`__rt_enum_exists` runtime helpers.
//!
//! Called from:
//! - `crate::codegen_ir::emit_program()` when the module's
//!   `const_introspection` runtime feature is set.
//!
//! Key details:
//! - Table layouts are link-time ABI shared with `__rt_sorted_name_search` and
//!   `__rt_constant`. Entries are sorted by name bytes so the helpers can binary
//!   search with `__rt_strcmp` ordering.
//! - The constant table stores a pre-boxed Mixed shape `(tag, lo, hi)` per entry
//!   so `__rt_constant` can reconstruct an owned Mixed via `__rt_mixed_from_value`
//!   (tags: int=0, string=1, float=2, bool=3, null=8).
//! - Class constants, enum-case singletons, array/composite constant values, and
//!   runtime `define()` overlays are intentionally absent (Stage 0 deferral): a
//!   miss is correct PHP behavior (`false` / thrown `\Error`).

use crate::ir::ConstScalar;

use super::instanceof::escaped_ascii;

/// Mixed tag for a boxed integer payload.
const MIXED_TAG_INT: i64 = 0;
/// Mixed tag for a boxed string payload (`lo` = bytes pointer, `hi` = length).
const MIXED_TAG_STRING: i64 = 1;
/// Mixed tag for a boxed float payload (`lo` = IEEE-754 bit pattern).
const MIXED_TAG_FLOAT: i64 = 2;
/// Mixed tag for a boxed boolean payload.
const MIXED_TAG_BOOL: i64 = 3;
/// Mixed tag for a boxed null payload.
const MIXED_TAG_NULL: i64 = 8;

/// Emits the constant/enum registry `.data` tables plus the shared `\Error`
/// class-id and "Undefined constant" message fragments used by `__rt_constant`.
///
/// `const_registry` is the name-sorted scalar constant list (canonical FQNs, no
/// leading `\`); `enum_names` is the name-sorted list of declared enum FQNs;
/// `error_class_id` is the runtime class id of `\Error` for this program (or
/// `u64::MAX` when `\Error` is not in scope, in which case a miss still unwinds
/// but cannot be matched by a user `catch (\Error)`).
pub(crate) fn emit_const_registry_data(
    const_registry: &[(String, ConstScalar)],
    enum_names: &[String],
    error_class_id: u64,
) -> String {
    let mut out = String::new();
    out.push_str(".data\n");
    out.push_str(".p2align 3\n");

    // -- constant table: fixed 40-byte entries {name_ptr, name_len, tag, lo, hi} --
    out.push_str(".globl _const_table_count\n_const_table_count:\n");
    out.push_str(&format!("    .quad {}\n", const_registry.len()));
    out.push_str(".globl _const_table\n_const_table:\n");
    for (index, (name, value)) in const_registry.iter().enumerate() {
        let (tag, lo, hi) = entry_tag_lo_hi(index, value);
        out.push_str(&format!("    .quad _constreg_name_{}\n", index));
        out.push_str(&format!("    .quad {}\n", name.len()));
        out.push_str(&format!("    .quad {}\n", tag));
        out.push_str(&format!("    .quad {}\n", lo));
        out.push_str(&format!("    .quad {}\n", hi));
    }
    for (index, (name, value)) in const_registry.iter().enumerate() {
        out.push_str(&format!("_constreg_name_{}:\n", index));
        out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii(name)));
        if let ConstScalar::Str(bytes) = value {
            out.push_str(&format!("_constreg_str_{}:\n", index));
            out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii(bytes)));
        }
    }

    // -- enum table: fixed 16-byte entries {name_ptr, name_len} --
    // Re-align: the preceding interned name bytes leave the location counter at
    // an arbitrary offset, but the following `.quad` pointers must be 8-aligned.
    out.push_str(".p2align 3\n");
    out.push_str(".globl _enum_table_count\n_enum_table_count:\n");
    out.push_str(&format!("    .quad {}\n", enum_names.len()));
    out.push_str(".globl _enum_table\n_enum_table:\n");
    for (index, name) in enum_names.iter().enumerate() {
        out.push_str(&format!("    .quad _constreg_enum_name_{}\n", index));
        out.push_str(&format!("    .quad {}\n", name.len()));
    }
    for (index, name) in enum_names.iter().enumerate() {
        out.push_str(&format!("_constreg_enum_name_{}:\n", index));
        out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii(name)));
    }

    // -- shared throw inputs for `constant()` on a registry miss --
    out.push_str(".p2align 3\n");
    out.push_str(".globl _constant_error_class_id\n_constant_error_class_id:\n");
    out.push_str(&format!("    .quad {}\n", error_class_id));
    out.push_str(".globl _constant_undefined_msg_prefix\n_constant_undefined_msg_prefix:\n");
    out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii("Undefined constant \"")));
    out.push_str(".globl _constant_undefined_msg_suffix\n_constant_undefined_msg_suffix:\n");
    out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii("\"")));

    out
}

/// Returns the boxed Mixed `(tag, lo, hi)` triple for a constant table entry.
///
/// Scalars are encoded so `__rt_constant` can hand them directly to
/// `__rt_mixed_from_value`: integers/bools/null carry their payload in `lo`,
/// floats carry their IEEE-754 bit pattern in `lo`, and strings carry a pointer
/// to the interned bytes in `lo` with the byte length in `hi`.
fn entry_tag_lo_hi(index: usize, value: &ConstScalar) -> (i64, String, i64) {
    match value {
        ConstScalar::Int(v) => (MIXED_TAG_INT, v.to_string(), 0),
        ConstScalar::Float(v) => (MIXED_TAG_FLOAT, v.to_bits().to_string(), 0),
        ConstScalar::Bool(v) => (MIXED_TAG_BOOL, i64::from(*v).to_string(), 0),
        ConstScalar::Str(bytes) => {
            (MIXED_TAG_STRING, format!("_constreg_str_{}", index), bytes.len() as i64)
        }
        ConstScalar::Null => (MIXED_TAG_NULL, "0".to_string(), 0),
    }
}
