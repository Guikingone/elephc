//! Purpose:
//! Records the interpreter's live call frames so `debug_backtrace()` can describe them.
//!
//! Called from:
//! - `crate::interpreter::dynamic_functions` and `crate::interpreter::statements` around each
//!   eval-declared function, closure, method, and static-method activation.
//! - `crate::interpreter::builtins::registry` around the class-probe builtins.
//! - `crate::interpreter::builtins::core::debug_backtrace` when a program asks for the trace.
//!
//! Key details:
//! - PHP describes a frame by the CALLEE's name and the CALLER's position: `function` is the
//!   function being entered, while `file` and `line` are where the call was written. Both halves
//!   are captured at the same moment, before the callee overrides the source metadata.
//! - A frame carries argument values by handle and does NOT retain them. A frame lives strictly
//!   inside the call it describes, so the caller's own references outlive it; retaining here would
//!   keep every argument of every live call alive for no reader.
//! - `args` is an `Option` because one caller genuinely cannot supply it: the direct builtin
//!   dispatch path hands the builtin unevaluated expressions, so no handle exists yet. PHP omits
//!   the `args` key under `DEBUG_BACKTRACE_IGNORE_ARGS`, so an omitted key is a shape PHP itself
//!   produces — an empty `args` array would instead be a wrong value.
//! - Builtin probes that can run an autoloader push frames too. That is not decoration: Symfony's
//!   `ClassExistenceResource::throwOnRequiredClass` reads `$trace[1]` and returns silently when
//!   that frame's `function` is one of the class probes and it has NO `class` key. Without those
//!   frames the same code throws `ReflectionException` instead.

use super::*;
use std::cell::RefCell;

/// How one call was written, which PHP reports as the frame's `type`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvalCallFrameKind {
    /// A plain function or closure call, which has neither `class` nor `type`.
    Function,
    /// An instance method call, reported as `->` with the receiver in `object`.
    Method,
    /// A static method call, reported as `::` with no `object`.
    StaticMethod,
}

/// One live interpreter call.
pub struct EvalCallFrame {
    /// PHP's `function` key: the unqualified name of the callee.
    pub function: String,
    /// PHP's `class` key, absent for a plain function.
    pub class: Option<String>,
    /// Selects PHP's `type` key and whether `object` can be present.
    pub kind: EvalCallFrameKind,
    /// PHP's `object` key for an instance call.
    pub object: Option<RuntimeCellHandle>,
    /// PHP's `args` key, or `None` when the call site could not supply the values.
    pub args: Option<Vec<RuntimeCellHandle>>,
    /// PHP's `file` key: the source file the call was written in.
    pub file: String,
    /// PHP's `line` key: the caller's source line.
    pub line: i64,
}

impl EvalCallFrame {
    /// Builds a plain-function frame positioned at the context's current call site.
    pub fn function(
        name: impl Into<String>,
        args: Option<Vec<RuntimeCellHandle>>,
        context: &ElephcEvalContext,
    ) -> Self {
        let (file, line) = eval_call_frame_position(context);
        Self {
            function: name.into(),
            class: None,
            kind: EvalCallFrameKind::Function,
            object: None,
            args,
            file,
            line,
        }
    }

    /// Builds a method frame positioned at the context's current call site.
    pub fn method(
        class_name: &str,
        method_name: impl Into<String>,
        object: Option<RuntimeCellHandle>,
        args: Option<Vec<RuntimeCellHandle>>,
        context: &ElephcEvalContext,
    ) -> Self {
        let (file, line) = eval_call_frame_position(context);
        Self {
            function: method_name.into(),
            class: Some(class_name.trim_start_matches('\\').to_string()),
            kind: match object {
                Some(_) => EvalCallFrameKind::Method,
                None => EvalCallFrameKind::StaticMethod,
            },
            object,
            args,
            file,
            line,
        }
    }
}

/// Reads the caller's source file and line for a frame about to be pushed.
///
/// `file` is exact: the interpreter overrides it per included file and per class method. `line` is
/// the context's current call-site line, which is the closest thing the eval IR carries — no
/// `EvalStmt` or `EvalExpr` records the line it was written on, so a per-statement line is not
/// available to this crate.
fn eval_call_frame_position(context: &ElephcEvalContext) -> (String, i64) {
    let (file, _, line, _) = context.call_site();
    let file = if context.eval_file_magic().is_empty() {
        file
    } else {
        context.eval_file_magic()
    };
    (file, line)
}

/// Returns whether PHP shows this builtin as its own `debug_backtrace()` frame here.
///
/// PHP shows every internal function it calls, but only this family is load-bearing: these are the
/// probes that can run an autoloader, so they are the frames a userland autoloader observes above
/// itself. The list is exactly the one `ClassExistenceResource::throwOnRequiredClass` switches on,
/// plus the two SPL entry points that reach the same callbacks.
pub fn eval_builtin_is_backtrace_visible(name: &str) -> bool {
    matches!(
        name,
        "class_exists"
            | "class_implements"
            | "class_parents"
            | "class_uses"
            | "defined"
            | "enum_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_parent_class"
            | "interface_exists"
            | "is_a"
            | "is_callable"
            | "is_subclass_of"
            | "method_exists"
            | "property_exists"
            | "spl_autoload"
            | "spl_autoload_call"
            | "trait_exists"
    )
}

thread_local! {
    /// PHP has ONE call stack per request; elephc had one per eval context.
    ///
    /// That difference is not academic. The interpreter runs an autoload callback in the context
    /// that REGISTERED it, not in the one that asked, so a loader calling `debug_backtrace()` saw
    /// only its own frame: the `class_exists()` that started the chain had been recorded on a
    /// different context and was invisible. Symfony's
    /// `ClassExistenceResource::throwOnRequiredClass` reads exactly that frame -- `$trace[1]`,
    /// whose `function` is one of the class probes and which has no `class` key -- and when it is
    /// missing the same code throws a `ReflectionException` nobody catches.
    ///
    /// A request runs on one thread and a forked web worker gets its own copy, so thread-local is
    /// the request's own stack. Pushes and pops are paired by the callers that own them.
    static EVAL_CALL_FRAMES: RefCell<Vec<EvalCallFrame>> = const { RefCell::new(Vec::new()) };
}

impl ElephcEvalContext {
    /// Records one call as entered.
    pub fn push_call_frame(&mut self, frame: EvalCallFrame) {
        EVAL_CALL_FRAMES.with(|frames| frames.borrow_mut().push(frame));
    }

    /// Records one call as left.
    pub fn pop_call_frame(&mut self) {
        EVAL_CALL_FRAMES.with(|frames| {
            frames.borrow_mut().pop();
        });
    }

    /// Reads the live frames, innermost last, for the backtrace builder to reverse.
    pub fn with_call_frames<R>(&self, read: impl FnOnce(&[EvalCallFrame]) -> R) -> R {
        EVAL_CALL_FRAMES.with(|frames| read(&frames.borrow()))
    }
}
