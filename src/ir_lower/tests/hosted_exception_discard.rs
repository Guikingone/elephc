//! Purpose:
//! Verifies that finally-discard notifications cross only the hosted PHP extension boundary.
//!
//! Called from:
//! - `crate::ir_lower::tests`.
//!
//! Key details:
//! - Every supported target emits the C adapter callback for `Emit::Phpext`.
//! - Ordinary native libraries keep the backend-neutral notification as a no-op.

use std::collections::HashMap;

use crate::codegen::{Emit, Instrumentation, WebIsolation};

/// Hosted user bodies emit discard callbacks on every target without changing cdylibs.
#[test]
fn finally_discard_notifies_only_php_extensions_on_all_targets() {
    let source = r#"<?php
function discard_pending(array $value): int {
    try { echo $value['missing']; } finally { return 1; }
}
"#;
    for target in [
        "macos-aarch64",
        "ios-arm64",
        "ios-sim-arm64",
        "linux-aarch64",
        "linux-x86_64",
    ] {
        let module = super::lower_source_at_for_target(
            source,
            std::path::Path::new("main.php"),
            std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        let emit = |kind| {
            crate::codegen::generate_user_asm_from_ir_with_options_and_features(
                &module,
                false,
                false,
                Instrumentation::Off,
                false,
                false,
                false,
                kind,
                &HashMap::new(),
                true,
                false,
                WebIsolation::Worker,
            )
            .unwrap()
            .asm
        };
        assert!(
            emit(Emit::Phpext).contains("elephc_php_discard_exception"),
            "{target}: the hosted discard callback is missing"
        );
        assert!(
            !emit(Emit::Cdylib).contains("elephc_php_discard_exception"),
            "{target}: an ordinary cdylib references the PHP adapter"
        );
    }
}
