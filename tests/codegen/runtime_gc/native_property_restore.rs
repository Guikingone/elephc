//! Purpose:
//! Cross-checks trusted native property name restoration against PHP's SPL loading protocol.
//!
//! Called from:
//! - Runtime GC integration tests through a private fixture AST transformation.
//!
//! Key details:
//! - PHP uses its real inherited __unserialize; only the oracle adapter is replaced for AOT.
//! - The adapter exposes pending destructor errors and rethrows readonly validation errors.
//! - Complete output and clean heaps are checked with both EIR optimizer modes.

use crate::support::*;
use elephc::names::{CompilerCall, Name};
use elephc::parser::ast::{Expr, ExprKind, Program, StmtKind};

/// Selects the internal name-based primitive without publishing a PHP builtin.
fn inject_restore(program: &mut Program) {
    let mut count = 0;
    for statement in program {
        let StmtKind::FunctionDecl { name, body, .. } = &mut statement.kind else { continue; };
        if name != "native_restore" { continue; }
        let mut result = body.pop().expect("oracle wrapper return");
        let StmtKind::Return(Some(value)) = &mut result.kind else { panic!("oracle return"); };
        value.kind = ExprKind::FunctionCall {
            name: Name::compiler_call(CompilerCall::NativePropertyRestore),
            args: ["h", "name", "value"].into_iter()
                .map(|name| Expr::new(ExprKind::Variable(name.into()), value.span)).collect(),
        };
        *body = vec![result];
        count += 1;
    }
    assert_eq!(count, 1);
}

/// Executes both native optimizer modes, or explicitly exports target assembly for the verifier.
fn assert_native_restore(source: &str, expected: &str) {
    let source = format!("<?php {} {source}", WRAPPER);
    let run = |mode: &str| with_fixture_ast_transform(inject_restore, || {
        if let Some(output) = std::env::var_os("ELEPHC_NATIVE_RESTORE_ASM_DIR") {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("native_restore").rsplit("::").next().unwrap();
            let dir = std::path::PathBuf::from(output).join(name).join(mode);
            std::fs::create_dir_all(&dir).unwrap();
            let (user, runtime, libraries) = compile_source_to_asm_with_options(
                &source, &dir, 8_388_608, false, true,
            );
            std::fs::write(dir.join("user.s"), user).unwrap();
            std::fs::write(dir.join("runtime.s"), runtime).unwrap();
            std::fs::write(dir.join("source.php"), &source).unwrap();
            std::fs::write(dir.join("expected.stdout"), expected).unwrap();
            std::fs::write(dir.join("libraries.txt"), format!("{libraries:?}")).unwrap();
            return;
        }
        let output = compile_and_run_with_heap_debug(&source);
        assert!(output.success, "{}\n{}", output.stdout, output.stderr);
        assert_eq!(output.stdout, expected, "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    });
    run("on");
    without_ir_opt(|| run("off"));
}

const WRAPPER: &str = "function native_restore(H $h, string $name, mixed $value): mixed { try { $h->__unserialize([[$name => $value], ['flags' => 0, 'heap_elements' => []]]); } catch (Throwable $error) { $previous = $error->getPrevious(); if ($previous instanceof Error && str_starts_with($previous->getMessage(), 'Cannot modify readonly property ')) { throw $previous; } return $previous; } return null; } ";

/// Preserves native public type bypass.
#[test]
fn test_native_property_restore_public_type_bypass() {
    assert_native_restore("class H extends SplMaxHeap { public int $value = 1; } $h = new H(); native_restore($h, 'value', 'bad'); echo gettype($h->value), ':', $h->value;", "string:bad");
}

/// Preserves native protected aliases.
#[test]
fn test_native_property_restore_protected_aliases() {
    assert_native_restore("class H extends SplMaxHeap { protected int $value = 1; function read(): mixed { return $this->value; } } $h = new H(); native_restore($h, 'value', 'plain'); echo $h->read(), '|'; native_restore($h, \"\\0*\\0value\", 'mangled'); echo $h->read();", "plain|mangled");
}

/// Preserves native private ancestors.
#[test]
fn test_native_property_restore_private_ancestors() {
    assert_native_restore("class P extends SplMaxHeap { private int $value = 1; function parentValue(): mixed { return $this->value; } } class H extends P { private int $value = 2; function childValue(): mixed { return $this->value; } } $h = new H(); native_restore($h, \"\\0P\\0value\", 'parent'); native_restore($h, \"\\0H\\0value\", 'child'); echo $h->parentValue(), ':', $h->childValue();", "parent:child");
}

/// Preserves native readonly uninitialized.
#[test]
fn test_native_property_restore_readonly_uninitialized() {
    assert_native_restore("class H extends SplMaxHeap { public readonly int $value; } $h = new H(); native_restore($h, 'value', 'bad'); echo gettype($h->value), ':', $h->value;", "string:bad");
}

/// Preserves native readonly initialized.
#[test]
fn test_native_property_restore_readonly_initialized() {
    assert_native_restore("class H extends SplMaxHeap { public readonly int $value; function __construct() { $this->value = 1; } } $h = new H(); try { native_restore($h, 'value', 'bad'); } catch (Error $error) { echo $error->getMessage(), '|'; } echo $h->value;", "Cannot modify readonly property H::$value|1");
}

/// Preserves native destructor view.
#[test]
fn test_native_property_restore_destructor_view() {
    assert_native_restore("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { echo $this->h->value === $this ? 'old|' : 'bad|'; } } $h = new H(); $h->value = new D($h); native_restore($h, 'value', 'new'); echo $h->value;", "old|new");
}

/// Preserves native pending destructor.
#[test]
fn test_native_property_restore_pending_destructor() {
    assert_native_restore("class H extends SplMaxHeap { public mixed $value = null; } class D { function __destruct() { throw new Exception('drop'); } } $h = new H(); $h->value = new D(); $error = native_restore($h, 'value', 'new'); if ($error instanceof Throwable) { echo $error->getMessage(), '|'; } echo $h->value;", "drop|new");
}

/// Preserves native dynamic empty.
#[test]
fn test_native_property_restore_dynamic_empty() {
    assert_native_restore("#[AllowDynamicProperties] class H extends SplMaxHeap {} $h = new H(); native_restore($h, '', 'empty'); $view = $h->__serialize(); echo json_encode($view[0]);", "{\"\":\"empty\"}");
}

/// Preserves native dynamic mangled.
#[test]
fn test_native_property_restore_dynamic_mangled() {
    assert_native_restore("#[AllowDynamicProperties] class H extends SplMaxHeap {} $h = new H(); native_restore($h, \"\\0Other\\0missing\", 'other'); $view = $h->__serialize(); echo json_encode($view[0]);", "{\"\\u0000Other\\u0000missing\":\"other\"}");
}

/// Preserves native public foreign scope.
#[test]
fn test_native_property_restore_public_foreign_scope() {
    assert_native_restore("class H extends SplMaxHeap { public int $value = 1; } $h = new H(); native_restore($h, \"\\0Other\\0value\", 'bad'); echo gettype($h->value), ':', $h->value;", "string:bad");
}

/// Preserves native protected foreign scope.
#[test]
fn test_native_property_restore_protected_foreign_scope() {
    assert_native_restore("class H extends SplMaxHeap { protected int $value = 1; function read(): mixed { return $this->value; } } $h = new H(); native_restore($h, \"\\0Other\\0value\", 'bad'); echo $h->read();", "bad");
}

/// Preserves native private scope case.
#[test]
fn test_native_property_restore_private_scope_case() {
    assert_native_restore("class P extends SplMaxHeap { private int $value = 1; function parentValue(): mixed { return $this->value; } } class H extends P { private int $value = 2; function childValue(): mixed { return $this->value; } } $h = new H(); native_restore($h, \"\\0p\\0value\", 'parent'); native_restore($h, \"\\0h\\0value\", 'child'); echo $h->parentValue(), ':', $h->childValue();", "parent:child");
}

/// Preserves native private plain dynamic.
#[test]
fn test_native_property_restore_private_plain_dynamic() {
    assert_native_restore("#[AllowDynamicProperties] class H extends SplMaxHeap { private int $value = 1; } $h = new H(); native_restore($h, 'value', 'shadow'); $view = $h->__serialize(); echo json_encode($view[0]);", "{\"\\u0000H\\u0000value\":1,\"value\":\"shadow\"}");
}

/// Preserves native private wrong scope.
#[test]
fn test_native_property_restore_private_wrong_scope() {
    assert_native_restore("#[AllowDynamicProperties] class H extends SplMaxHeap { private int $value = 1; } $h = new H(); native_restore($h, \"\\0Other\\0value\", 'shadow'); $view = $h->__serialize(); echo json_encode($view[0]);", "{\"\\u0000H\\u0000value\":1,\"\\u0000Other\\u0000value\":\"shadow\"}");
}
