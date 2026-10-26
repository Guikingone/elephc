//! Purpose:
//! Shares the key-pair extension used until a boxed receiver chooses its PHP key rules.
//!
//! Called from:
//! - EIR Mixed array lowering and indexed/hash Mixed receiver runtime branches.
//!
//! Key details:
//! - Integer keys retain high word -1 and strings retain a nonnegative byte length.
//! - A high word below -1 encodes -(runtime tag + 2), with the original borrowed payload.
//! - Only actual array branches reject these keys; object callbacks recover the original value.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

pub(crate) const DEFERRED_KEY_TAG_BIAS: i64 = 2;
pub(crate) const ARRAY_OFFSET_TYPE_SUFFIX: &str = " on array";

/// Encodes an original key type without acquiring a temporary boxed owner.
pub(crate) const fn deferred_key_marker(tag: i64) -> i64 {
    -(tag + DEFERRED_KEY_TAG_BIAS)
}

/// Rejects an original non-scalar key only after the receiver selected an array branch.
/// The runtime getter/setter frame keeps its original key at the common ABI offsets.
pub(crate) fn emit_array_key_guard(emitter: &mut Emitter, label: &str, consumed_value: bool) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emitter.instruction("ldr x9, [sp, #16]");                           // inspect the original key type carried in the high word
            emitter.instruction("cmn x9, #1");                                  // integers and string lengths are the valid normalized forms
            emitter.instruction(&format!("b.ge {label}"));                      // preserve the receiver and key registers on the ordinary path
            if consumed_value {
                emitter.instruction("ldr x0, [sp, #24]");                       // select the setter's incoming owned value before throwing
                abi::emit_call_label(emitter, "__rt_decref_mixed");
            }
            emitter.instruction("ldr x1, [sp, #8]");                            // pass the borrowed original key payload
            emitter.instruction("ldr x2, [sp, #16]");                           // pass the deferred runtime key tag
        }
        Arch::X86_64 => {
            emitter.instruction("cmp QWORD PTR [rbp - 24], -1");                // integers and string lengths are the valid normalized forms
            emitter.instruction(&format!("jge {label}"));                       // preserve the receiver and key registers on the ordinary path
            if consumed_value {
                emitter.instruction("mov rax, QWORD PTR [rbp - 32]");           // select the setter's incoming owned value before throwing
                abi::emit_call_label(emitter, "__rt_decref_mixed");
            }
            emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");               // pass the borrowed original key payload
            emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");               // pass the deferred runtime key tag
        }
    }
    abi::emit_call_label(emitter, "__rt_throw_deferred_array_key");
    emitter.label(label);
}
