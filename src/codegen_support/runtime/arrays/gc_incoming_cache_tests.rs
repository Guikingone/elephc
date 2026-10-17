//! Purpose:
//! Checks the collector's target-wide edge-cache and eval-child ABI emission.
//!
//! Called from:
//! - The runtime array library test module.
//!
//! Key details:
//! - x86 packed counts saturate below the heap marker and retain exact fallback.
//! - Every supported target defines the all-edge eval sibling without duplicate labels.

use crate::codegen_support::{emit::Emitter, platform::Target};
use super::{emit_gc_collect_cycles, emit_gc_note_child_ref};
use super::gc_eval_object_children::emit_gc_eval_object_children;

/// Every ABI emits a distinct count-all entry, with unique labels in direct and PIC modes.
#[test]
fn gc_incoming_cache_all_targets_define_unique_edge_helpers() {
    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        for pic in [false, true] {
            let mut emitter = Emitter::new(Target::parse(name).unwrap());
            emitter.pic_data_refs = pic;
            emit_gc_collect_cycles(&mut emitter);
            emit_gc_note_child_ref(&mut emitter);
            emit_gc_eval_object_children(&mut emitter);
            let assembly = emitter.output();
            let mut labels = std::collections::HashSet::new();
            for label in assembly.lines().filter_map(|line| line.trim().strip_suffix(':')) {
                if label.chars().all(|ch| ch.is_ascii_digit()) { continue; }
                assert!(labels.insert(label), "{name}, pic={pic}: duplicate label {label}");
            }
            assert!(labels.contains("__rt_gc_eval_object_children_count_all"), "{name}, pic={pic}");
            if name == "linux-x86_64" {
                assert!(assembly.contains("__rt_gc_collect_cycles_cache_init:"));
                assert!(assembly.contains("cmp QWORD PTR [rbp - 48], 8191"));
                assert!(assembly.contains("jne __rt_gc_collect_cycles_root_compare"));
                assert!(assembly.contains("add DWORD PTR [rax - 8], 524288"));
                assert!(assembly.contains("cmp r10d, 8191"));
                assert!(assembly.contains("mov rcx, 0xffffffff0007ffff"));
            } else {
                assert!(!assembly.contains("__rt_gc_collect_cycles_cache_init:"));
                assert!(assembly.contains("bl __rt_gc_note_child_ref"));
            }
        }
    }
}
