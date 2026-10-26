//! Purpose:
//! Freezes native property replacement order through a trusted compiler-call test bridge.
//!
//! Called from:
//! - Runtime GC integration tests with the fixture AST transform.
//!
//! Key details:
//! - Reference PHP executes the actual inherited __unserialize protocol.
//! - Only the test helper body becomes the private replacement primitive; no PHP API is registered.
//! - Both optimizer modes check complete output and ownership, including destructor reentry.

use crate::support::*;
use elephc::names::{CompilerCall, Name};
use elephc::parser::ast::{Expr, ExprKind, Program, StmtKind};

/// Replaces only the native oracle wrapper, preserving all user callbacks and field expressions.
fn inject_replace(program: &mut Program) {
    let mut count = 0;
    for statement in program {
        let StmtKind::FunctionDecl { name, body, .. } = &mut statement.kind else { continue; };
        if name != "native_replace" { continue; }
        let mut result = body.pop().expect("oracle wrapper return");
        let StmtKind::Return(Some(value)) = &mut result.kind else { panic!("oracle return"); };
        value.kind = ExprKind::FunctionCall {
            name: Name::compiler_call(CompilerCall::NativePropertyReplace),
            args: vec![
                Expr::new(ExprKind::Variable("h".into()), value.span),
                Expr::new(ExprKind::IntLiteral(72), value.span),
                Expr::new(ExprKind::Variable("value".into()), value.span),
            ],
        };
        *body = vec![result];
        count += 1;
    }
    assert_eq!(count, 1);
}

/// Runs a full frontend rewrite and both native EIR modes with an actual PHP oracle wrapper.
fn assert_native_view(source: &str, expected: &str) {
    let source = format!("<?php function native_replace(H $h, mixed $value): mixed {{ try {{ $h->__unserialize([['value' => $value], ['flags' => 0, 'heap_elements' => []]]); }} catch (Throwable $error) {{ return $error->getPrevious(); }} return null; }} {source}");
    let run = |mode: &str| with_fixture_ast_transform(inject_replace, || {
        if let Some(output) = std::env::var_os("ELEPHC_NATIVE_VIEW_ASM_DIR") {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("native_view").rsplit("::").next().unwrap();
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

/// A native loader stores the actual incoming tag independently from an int declaration.
#[test]
fn test_native_property_view_declared_type_bypass() {
    assert_native_view("class H extends SplMaxHeap { public int $value = 1; } $h = new H(); native_replace($h, 'bad'); echo gettype($h->value), ':', $h->value;", "string:bad");
}

/// A retiring object remains the logical property value throughout its own destructor.
#[test]
fn test_native_property_view_object_destructor() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { echo $this->h->value === $this ? 'old|' : 'changed|'; } } $h = new H(); $h->value = new D($h); native_replace($h, 'new'); echo $h->value;", "old|new");
}

/// Destructor failure still publishes the incoming value and returns its pending exception.
#[test]
fn test_native_property_view_pending_destructor_error() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { function __destruct() { throw new Exception('drop'); } } $h = new H(); $h->value = new D(); $error = native_replace($h, 'new'); if ($error instanceof Throwable) { echo $error->getMessage(), '|'; } echo $h->value;", "drop|new");
}

/// A copied old value can resurrect its object without postponing the original destructor.
#[test]
fn test_native_property_view_resurrection() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { global $saved; echo 'drop|'; $saved = $this->h->value; } } $saved = null; $h = new H(); $d = new D($h); $weak = WeakReference::create($d); $h->value = $d; unset($d); native_replace($h, 'new'); echo $weak->get() === $saved ? 'alive|' : 'bad|'; $saved = null; echo $weak->get() === null ? 'gone' : 'bad';", "drop|alive|gone");
}

/// A nested destructor sees the old array rather than the incoming scalar.
#[test]
fn test_native_property_view_array_destructor() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { echo is_array($this->h->value) ? 'array|' : 'changed|'; } } $h = new H(); $h->value = [new D($h)]; native_replace($h, 'new'); echo $h->value;", "array|new");
}

/// Preserves the complete pinned PHP view for array contents.
#[test]
fn test_native_property_view_array_contents() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; echo $view[0] === $this ? 'old:' : 'bad:'; echo $view[1], ':', $view[2], ':', $view[3][0], '|'; } } $h = new H(); $h->value = [new D($h), 'text', 7, [4]]; native_replace($h, 'new'); echo $h->value;", "old:text:7:4|new");
}

/// Preserves the complete pinned PHP view for nested array.
#[test]
fn test_native_property_view_nested_array() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; echo $view[0][0] === $this ? 'nested:' : 'bad:'; echo $view[1], '|'; } } $h = new H(); $h->value = [[new D($h)], 'text']; native_replace($h, 'new'); echo $h->value;", "nested:text|new");
}

/// Preserves the pinned PHP old-value clone through the clone static path.
#[test]
fn test_native_property_view_clone_static() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { global $copy; $copy = clone $this->h; echo $copy->value === $this ? 'old|' : 'bad|'; } } $copy = new H(); $h = new H(); $h->value = new D($h); native_replace($h, 'new'); $copy = $copy; if ($copy instanceof H) { echo $h->value, ':', gettype($copy->value); }", "old|new:object");
}

/// Preserves the pinned PHP old-value clone through the clone boxed path.
#[test]
fn test_native_property_view_clone_boxed() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public mixed $h; function __construct(H $h) { $this->h = $h; } function __destruct() { global $copy; $copy = clone $this->h; echo $copy->value === $this ? 'old|' : 'bad|'; } } $copy = new H(); $h = new H(); $h->value = new D($h); native_replace($h, 'new'); $copy = $copy; if ($copy instanceof H) { echo $h->value, ':', gettype($copy->value); }", "old|new:object");
}

/// Preserves the pinned PHP borrowed view for hash contents.
#[test]
fn test_native_property_view_hash_contents() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; echo $view['self'] === $this ? 'old:' : 'bad:'; echo $view['text'], ':', $view['nested'][0], '|'; } } $copy = new H(); $h = new H(); $h->value = ['self' => new D($h), 'text' => 'text', 'nested' => [4]]; native_replace($h, 'new'); echo $h->value;", "old:text:4|new");
}

/// Preserves the pinned PHP borrowed view for hash nested.
#[test]
fn test_native_property_view_hash_nested() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; echo $view['nested'][0]['self'] === $this ? 'nested:' : 'bad:'; echo $view['text'], '|'; } } $copy = new H(); $h = new H(); $h->value = ['nested' => [['self' => new D($h)]], 'text' => 'text']; native_replace($h, 'new'); echo $h->value;", "nested:text|new");
}

/// Preserves the pinned PHP borrowed view for array nested hash.
#[test]
fn test_native_property_view_array_nested_hash() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; echo $view[0]['self'] === $this ? 'nested:' : 'bad:'; echo $view[1], '|'; } } $copy = new H(); $h = new H(); $h->value = [['self' => new D($h)], 'text']; native_replace($h, 'new'); echo $h->value;", "nested:text|new");
}

/// Preserves the pinned PHP borrowed view for hash key order.
#[test]
fn test_native_property_view_hash_key_order() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { $view = $this->h->value; foreach ($view as $key => $value) { echo gettype($key), ':', $key, ';'; } echo '|'; } } $copy = new H(); $h = new H(); $h->value = ['self' => new D($h), 8 => 'eight', '08' => 'zero-eight', -3 => 'negative', '' => 'empty']; native_replace($h, 'new'); echo $h->value;", "string:self;integer:8;string:08;integer:-3;string:;|new");
}

/// Preserves the pinned PHP borrowed view for hash clone.
#[test]
fn test_native_property_view_hash_clone() {
    assert_native_view("class H extends SplMaxHeap { public mixed $value = null; } class D { public H $h; function __construct(H $h) { $this->h = $h; } function __destruct() { global $copy; $copy = clone $this->h; $view = $copy->value; echo $view['self'] === $this ? 'old:' : 'bad:'; echo $view['text'], '|'; } } $copy = new H(); $h = new H(); $h->value = ['self' => new D($h), 'text' => 'text']; native_replace($h, 'new'); echo $h->value;", "old:text|new");
}
