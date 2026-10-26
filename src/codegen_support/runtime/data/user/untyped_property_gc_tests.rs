//! Purpose:
//! Pins physical Mixed metadata for untyped properties without defaults.
//!
//! Called from:
//! - The runtime user-data unit test module.
//!
//! Key details:
//! - Logical Void still occupies an owning boxed Mixed property slot.
//! - GC and JSON descriptors must agree on every supported target.

use super::{emit_runtime_data_user, tests::empty_class_info};
use crate::codegen_support::platform::Target;
use crate::types::PhpType;
use std::collections::{HashMap, HashSet};

#[test]
fn untyped_property_gc_descriptors_use_mixed_on_all_targets() {
    let mut info = empty_class_info(1, "run");
    info.properties.push(("value".to_string(), PhpType::Void));
    info.property_offsets.insert("value".to_string(), 8);
    info.properties.push(("count".to_string(), PhpType::Int));
    info.property_offsets.insert("count".to_string(), 24);
    let classes = HashMap::from([("UntypedOwner".to_string(), info)]);

    for name in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let asm = emit_runtime_data_user(
            &HashSet::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashSet::new(),
            &HashMap::new(),
            &[],
            &[],
            &HashMap::new(),
            &HashMap::new(),
            &classes,
            &HashMap::new(),
            &HashSet::new(),
            None,
            false,
            None,
            Target::parse(name).unwrap(),
        ).unwrap();

        assert!(asm.contains("_class_gc_desc_1:\n    .byte 7, 0\n"), "{name}");
        assert!(asm.contains(concat!(
            "_class_json_desc_1:\n",
            "    .quad 0\n    .quad 0\n    .quad 2\n",
            "    .quad _class_json_pname_1_0\n",
            "    .quad 5\n    .quad 0\n    .quad 7\n",
        )), "{name}");
    }
}
