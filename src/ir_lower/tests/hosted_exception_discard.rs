//! Purpose:
//! Verifies that exception-settlement notifications cross only the hosted PHP extension boundary.
//!
//! Called from:
//! - `crate::ir_lower::tests`.
//!
//! Key details:
//! - Every supported target emits discard and caught-replacement callbacks for `Emit::Phpext`.
//! - Ordinary native libraries keep the backend-neutral notification as a no-op.

use std::collections::HashMap;

use crate::codegen::{Emit, Instrumentation, WebIsolation};
use crate::ir::{Immediate, Op, RuntimeCallTarget};

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
        let hosted = emit(Emit::Phpext);
        let visit = hosted
            .find("discard_host_replacement_visit")
            .expect("the hosted discard traversal is missing");
        let callback = hosted[visit..]
            .find("elephc_php_discard_exception")
            .expect("the hosted discard callback is missing");
        let previous = hosted[visit..]
            .find("__rt_throwable_previous")
            .expect("the hosted discard chain walk is missing");
        assert!(
            callback < previous,
            "{target}: discarded wrappers must traverse their complete native previous chain"
        );
        assert!(
            hosted.contains("elephc_php_discard_exception"),
            "{target}: the hosted discard callback is missing"
        );
        assert!(
            !emit(Emit::Cdylib).contains("elephc_php_discard_exception"),
            "{target}: an ordinary cdylib references the PHP adapter"
        );
    }
}

/// User catches mark native replacements only for the hosted PHP adapter on every target.
#[test]
fn caught_host_replacements_notify_only_php_extensions_on_all_targets() {
    let source = r#"<?php
function catch_replacement(array $value): void {
    try { try { echo $value['missing']; } finally { throw new RuntimeException('native'); } }
    catch (RuntimeException $error) { throw new TypeError('after'); }
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
        let function = module
            .functions
            .iter()
            .find(|function| function.name == "catch_replacement")
            .expect("expected catch_replacement EIR function");
        assert!(function.instructions.iter().any(|inst| {
            inst.op == Op::RuntimeCall
                && inst.immediate
                    == Some(Immediate::RuntimeCall(
                        RuntimeCallTarget::ExceptionCatchHostReplacement,
                    ))
        }));
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
        let hosted = emit(Emit::Phpext);
        let callback = hosted
            .find("elephc_php_catch_host_replacement")
            .expect("the hosted catch notification is missing");
        assert!(
            hosted[callback..].contains("__rt_throwable_previous"),
            "{target}: caught wrappers must traverse their complete native previous chain"
        );
        assert!(
            hosted.contains("elephc_php_catch_host_replacement"),
            "{target}: the hosted catch notification is missing"
        );
        assert!(
            !emit(Emit::Cdylib).contains("elephc_php_catch_host_replacement"),
            "{target}: an ordinary cdylib references the PHP adapter"
        );
    }
}
