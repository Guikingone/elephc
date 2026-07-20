//! Purpose:
//! Injects the PHP `register_shutdown_function()` standard-library function (written in
//! elephc-PHP) that maintains a real, registration-ordered callback registry invoked once at
//! normal script end and once before `exit()`/`die()`, matching PHP's shutdown-function contract.
//!
//! Called from:
//! - `crate::pipeline::compile()` and the codegen test harness via `inject_if_used`, at the exact
//!   same pipeline stage as `crate::var_export_prelude` (after `autoload::run` and the
//!   conditional-function hoist, before `optimize::fold_constants`), so PSR-4 autoloaded usage is
//!   detected and the declaration is present before the type checker collects functions.
//! - `crate::codegen_ir::context::FunctionContext::emit_shutdown_functions_runner_call_if_present`
//!   reads `RUN_SHUTDOWN_FUNCTIONS_NAME` to conditionally call the injected runner from the
//!   top-level epilogue (`codegen_ir::frame::emit_main_epilogue`) and from `exit()`/`die()`
//!   lowering (`codegen_ir::lower_inst::builtins::system::lower_exit`) — the single source of
//!   truth for the runner's name keeps both sides in lockstep.
//!
//! Key details:
//! - Implemented as a prelude (not hand-written runtime assembly) so the registry is a REAL PHP
//!   array: registration is `$callbacks[] = ...;`, invocation is a plain `while` loop calling each
//!   entry. This reuses elephc's existing array/static-property/dynamic-call machinery and gets
//!   both supported targets for free — no new `__rt_*` assembly, no new EIR op.
//! - The registry is a STATIC CLASS PROPERTY (`__ElephcShutdownRegistry::$callbacks`), NOT a
//!   `global` array: an element read back from a `global`-declared array loses its element type
//!   across functions (inferred `Void`/`Never`), which either fails the checker or the EIR backend
//!   (`callable_descriptor_invoke for callable PHP type Void`). A `public static array` property
//!   element reads back as `Mixed`, whose no-argument dynamic-callable dispatch path is
//!   implemented and verified end-to-end. The same choice sidesteps the previously-documented
//!   `global`-array `mixed`-variadic corruption gate (wrong `count()`, empty element contents) —
//!   the variadic-collected `$args` array is never stored through a `global` slot at all.
//! - BOUND ARGUMENTS (`mixed ...$args`) ARE SUPPORTED, PHP-faithfully, by binding them at
//!   REGISTRATION time: `register_shutdown_function` immediately wraps `($callback, $args)` into a
//!   zero-argument closure via the free function `__elephc_shutdown_wrap`, so the runner only ever
//!   performs a no-argument dynamic call `$c()` on a `Mixed` element — the one dynamic-invoke
//!   shape proven to work through static-array storage. Probed-and-rejected alternatives: passing
//!   the stored `Mixed` element back into a `callable`/`array`-typed helper parameter loses the
//!   callable descriptor's invoker at runtime ("callable descriptor without invoker"); spreading a
//!   `Mixed` element is rejected by the checker; `call_user_func_array` on a `Mixed` callback is
//!   deliberately rejected by the checker (`src/types/checker/builtins/callables.rs`) because the
//!   EIR path behind that acceptance does not exist — relaxing it would trade a loud compile error
//!   for a runtime fatal.
//! - `__elephc_shutdown_wrap` must be a FREE function: the same wrapper closure created inline
//!   inside a static method invokes with its captured `$args` array empty (pre-existing
//!   static-method closure-capture bug, out of this feature's scope); created inside a free
//!   function, the captures survive storage and later `Mixed` dynamic dispatch intact.
//! - `register_shutdown_function` itself is declared with a REAL `callable $callback` parameter,
//!   so the ordinary checked-user-function call path validates it — the callback must already be
//!   `PhpType::Callable` (a closure, a first-class callable, or an existing `callable`-typed
//!   value), OR a LITERAL string naming a known function (user-declared or a builtin with a
//!   first-class-callable signature), which is coerced to a first-class-callable closure at
//!   COMPILE TIME by the generic `callable`-typed-parameter string coercion (see
//!   `crate::types::checker::functions::call_validation::Checker::coerce_callable_string_args`
//!   for the type-check-time acceptance and `crate::optimize::callable_coercion` for the real
//!   AST rewrite that makes `crate::ir_lower` see the same coerced node): PHP-visible
//!   `register_shutdown_function('someFunc')` now compiles and runs, matching PHP exactly (php
//!   -n verified). PHP additionally accepts a `[obj, 'method']` array callable and a
//!   `'Class::method'` static-method string; elephc's coercion does not extend to either form
//!   this cycle (array-form needs the receiver's runtime type; static-method-string visibility
//!   depends on the calling scope — neither drops out trivially from the string-literal seam),
//!   so those two remain loudly rejected with a type-mismatch diagnostic (JURY ADDENDUM #2 on
//!   the N1 checker-bundle spec: explicit decision, documented, not silent). Likewise, a
//!   NON-literal string (a variable holding a function name) and a call using named/spread
//!   arguments are not coerced (dynamic name resolution and call-shape reconstruction are both
//!   out of scope) — all documented, honest gaps: the call fails to type-check instead of
//!   silently miscompiling.
//! - Re-entry guard: `__ElephcShutdownRegistry::$running` is checked and set before the loop, so
//!   a shutdown callback that itself calls `exit()`/`die()` (which re-enters the epilogue/exit
//!   path and would otherwise re-invoke the runner) returns immediately instead of recursing.
//! - Registration order and "a shutdown function registering another" are both handled by the
//!   `while ($i < count(__ElephcShutdownRegistry::$callbacks))` loop re-reading the live array
//!   length each iteration (never caching it), so a callback that calls
//!   `register_shutdown_function()` again mid-run appends to the same registry and the loop picks
//!   it up before exiting — matching PHP, which also runs late-registered shutdown functions after
//!   the current queue.
//! - Fatal-error-triggered shutdown (PHP invokes registered shutdown functions on a fatal error,
//!   not just normal exit) is explicitly OUT of scope: elephc has no error-handling/fatal-signal
//!   infrastructure to hook this into. Only the two points spelled out in the PHP manual that
//!   elephc actually models — normal script end and `exit()`/`die()` — are covered.

use crate::parser::ast::Program;

mod detect;

/// The name of the internal runner function the prelude declares and the codegen hooks call by
/// symbol. Not PHP-visible under this name from user code in any meaningful way (the leading
/// `__elephc_` prefix is reserved), but kept as a named constant rather than a magic string so the
/// prelude source and the codegen call sites cannot drift apart.
pub(crate) const RUN_SHUTDOWN_FUNCTIONS_NAME: &str = "__elephc_run_shutdown_functions";

/// The elephc-PHP `register_shutdown_function` prelude: the public PHP-faithful
/// `register_shutdown_function(callable $callback, mixed ...$args)` entry point, a
/// registration-ordered registry (a static-property array of pre-bound zero-argument wrapper
/// closures plus a re-entry guard flag), and the internal runner
/// `__elephc_run_shutdown_functions()` that `codegen_ir` calls directly by symbol from the
/// top-level epilogue and from `exit()`/`die()` lowering. See the module-level "Key details" for
/// why the registry is a static class property and why arguments are bound at registration time.
pub const SHUTDOWN_PRELUDE_SRC: &str = r#"<?php
class __ElephcShutdownRegistry {
    public static array $callbacks = [];
    public static bool $running = false;
}

function __elephc_shutdown_wrap(callable $cb, array $args): callable {
    return function () use ($cb, $args): void {
        call_user_func_array($cb, $args);
    };
}

function register_shutdown_function(callable $callback, mixed ...$args): void {
    __ElephcShutdownRegistry::$callbacks[] = __elephc_shutdown_wrap($callback, $args);
}

function __elephc_run_shutdown_functions(): void {
    if (__ElephcShutdownRegistry::$running) {
        return;
    }
    __ElephcShutdownRegistry::$running = true;
    $i = 0;
    while ($i < count(__ElephcShutdownRegistry::$callbacks)) {
        $c = __ElephcShutdownRegistry::$callbacks[$i];
        $c();
        $i++;
    }
}
"#;

/// Prepends the shutdown-function prelude when the program references
/// `register_shutdown_function` and does not declare its own, so unrelated binaries pay nothing
/// and a user definition is not clobbered. The prelude is hoisted global-scope statements and
/// function declarations only, so prepending does not change top-level execution order for the
/// user program that follows it. The source is static and tested, so a tokenize/parse failure is a
/// compiler bug and panics rather than degrading silently.
pub fn inject_if_used(program: Program) -> Program {
    if !detect::program_references_register_shutdown_function(&program)
        || detect::program_declares_register_shutdown_function(&program)
    {
        return program;
    }
    let tokens = crate::lexer::tokenize(SHUTDOWN_PRELUDE_SRC)
        .expect("shutdown prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("shutdown prelude must parse");
    combined.extend(program);
    combined
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Function-level tests for the `inject_if_used` pay-for-use guard, covering the
    //! "only when used" contract and the user-declaration skip, mirroring
    //! `var_export_prelude`'s test shape at the same pipeline stage.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Source is parsed the way `inject_if_used` sees it: tokenize then parse.

    use super::*;

    /// Parses source the way `inject_if_used` sees it: tokenize then parse.
    fn parse(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// A program with no `register_shutdown_function` usage is returned unchanged.
    #[test]
    fn no_injection_when_unused() {
        let program = parse(r#"<?php $a = [1, 2]; echo count($a);"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }

    /// A program that calls `register_shutdown_function` gets the prelude prepended.
    #[test]
    fn injection_when_used() {
        let program = parse(r#"<?php register_shutdown_function(function () {});"#);
        let injected = inject_if_used(program.clone());
        assert!(injected.len() > program.len());
    }

    /// A program that declares its own `register_shutdown_function` is returned unchanged, so the
    /// user definition wins and there is no redeclaration conflict.
    #[test]
    fn no_injection_when_user_declares() {
        let program = parse(r#"<?php function register_shutdown_function($cb) {}"#);
        let injected = inject_if_used(program.clone());
        assert_eq!(injected.len(), program.len());
    }
}
