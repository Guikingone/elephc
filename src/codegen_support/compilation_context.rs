//! Purpose:
//! Stores per-thread compile profile, autoload metadata, and linked extension state.
//!
//! Called from:
//! - "crate::pipeline" before lowering and code generation.
//!
//! Key details:
//! - Thread-local storage isolates parallel compiler tests while keeping deep lowering APIs narrow.

use std::cell::{Cell, RefCell};

thread_local! {
    /// Number of `spl_autoload_register` closure rules extracted before code generation.
    static AUTOLOAD_RULE_COUNT: Cell<usize> = const { Cell::new(0) };
    /// Canonical PHP extension names supplied by linked bridge static libraries.
    static LINKED_EXTENSIONS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// Selected PHP language profile and whether the current compilation uses web SAPI.
    static COMPILE_PROFILE: Cell<(crate::web_prelude::PhpVersion, bool)> =
        const { Cell::new((crate::web_prelude::PhpVersion::Php85, false)) };
}

/// Records the PHP language profile and SAPI mode of the current compilation.
///
/// Called once from `pipeline::compile` before any prelude runs, so every later
/// phase (prescan constants, EIR lowering, per-instruction lowering) observes the
/// same pair. Read via [`compile_php_version`] / [`compile_is_web_sapi`].
pub fn set_compile_profile(php_version: crate::web_prelude::PhpVersion, web: bool) {
    COMPILE_PROFILE.with(|profile| profile.set((php_version, web)));
}

/// Returns the PHP language profile this compilation targets (default: the newest
/// maintained profile, matching `--php-version`'s own default).
pub(crate) fn compile_php_version() -> crate::web_prelude::PhpVersion {
    COMPILE_PROFILE.with(|profile| profile.get().0)
}

/// Returns whether this compilation is a `--web` build (default: `false`, i.e. CLI).
pub(crate) fn compile_is_web_sapi() -> bool {
    COMPILE_PROFILE.with(|profile| profile.get().1)
}

/// Sets the number of autoload rules registered.
pub fn set_autoload_rule_count(n: usize) {
    AUTOLOAD_RULE_COUNT.with(|c| c.set(n));
}

/// Returns the number of autoload rules registered.
pub fn autoload_rule_count() -> usize {
    AUTOLOAD_RULE_COUNT.with(|c| c.get())
}

/// Records the canonical PHP extension names of the bridge staticlibs linked
/// into the current compilation, so `extension_loaded()` / `get_loaded_extensions()`
/// report them in addition to the always-present core set. Set from the pipeline
/// before codegen; read via [`linked_extensions`]. Passing an empty vec (the
/// default) reports only the core extensions, matching a bridge-free program.
pub fn set_linked_extensions(extensions: Vec<String>) {
    LINKED_EXTENSIONS.with(|names| *names.borrow_mut() = extensions);
}

/// Returns the canonical PHP extension names of the bridges linked into the
/// current compilation (empty unless [`set_linked_extensions`] ran for it).
pub(crate) fn linked_extensions() -> Vec<String> {
    LINKED_EXTENSIONS.with(|names| names.borrow().clone())
}

thread_local! {
    /// Whether the file under compilation opened with `declare(strict_types=1)`.
    ///
    /// PHP requires that directive to be the very first statement, so the parser
    /// knows the answer before any type check runs and records it here. It rides
    /// the same thread-local channel as [`COMPILE_PROFILE`] for the same reason:
    /// the consumer is `require_compatible_arg_type`, buried under every call
    /// site in the checker, and `Program` is a bare `Vec<Stmt>` with nowhere to
    /// put it. The default is coercive typing, which is what a file without the
    /// directive — and every unit test — must observe.
    static STRICT_TYPES: Cell<bool> = const { Cell::new(false) };
}

/// Records whether the file under compilation declared `strict_types=1`.
///
/// Called by the parser when it accepts the directive, before type checking.
pub fn set_strict_types(strict: bool) {
    STRICT_TYPES.with(|strict_types| strict_types.set(strict));
}

/// Returns whether the current compilation uses PHP's strict typing mode.
///
/// Under strict typing PHP performs no scalar coercion at a typed parameter, with
/// the single documented exception of widening `int` to `float`.
pub(crate) fn strict_types() -> bool {
    STRICT_TYPES.with(|strict_types| strict_types.get())
}
