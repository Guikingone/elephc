//! Purpose:
//! Applies PHP's parameter-binding rules when a declared user-defined parameter is handed an
//! argument whose inferred type does not already satisfy it: coercive scalar binding
//! (`string $s` accepting `42`) and callable-name strings (`callable $f` accepting
//! `"strtoupper"`).
//!
//! Called from:
//! - `crate::types::checker::functions::resolution` (user function and method calls)
//!
//! Key details:
//! - The accept/reject decision lives in `crate::types::param_binding`, shared with the
//!   matching EIR argument rewrite. This file only turns that decision into a diagnostic and
//!   registers the callable signature a bound callable string implies.
//! - It runs *after* `types_compatible` / `type_accepts` have already failed, so it can only
//!   widen what is accepted, never narrow it.

use crate::errors::CompileError;
use crate::parser::ast::Expr;
use crate::types::param_binding::{classify_param_binding, ParamBinding};
use crate::types::{FunctionSig, PhpType, TypeEnv};

use super::super::Checker;

impl Checker {
    /// Validates one declared parameter against an argument, allowing PHP's coercive
    /// parameter binding and callable-name strings before reporting a type mismatch.
    ///
    /// `owner` names the `(function, parameter)` pair used to register the signature of a
    /// callable-name string, so the callee can type-check invocations of that parameter the
    /// same way it does for a first-class callable. Pass `None` where that registration does
    /// not apply (variadic elements, spread-expanded positions).
    ///
    /// `by_ref` must be true for a pass-by-reference parameter. PHP coerces those in place and
    /// writes the converted value back to the caller's variable; elephc's binding produces a
    /// temporary instead, so a by-reference parameter stays on the strict path rather than
    /// silently dropping the callee's writes.
    ///
    /// # Errors
    /// Returns the standard `<context> expects <expected>, got <actual>` mismatch, extended
    /// with the PHP behaviour elephc cannot reproduce when a binding rule exists but is not
    /// statically decidable.
    pub(crate) fn require_bound_param_arg_type(
        &mut self,
        expected: &PhpType,
        actual: &PhpType,
        arg: &Expr,
        env: &TypeEnv,
        context: &str,
        owner: Option<(&str, &str)>,
        by_ref: bool,
    ) -> Result<(), CompileError> {
        if Self::types_compatible(expected, actual) || self.type_accepts(expected, actual) {
            return Ok(());
        }
        if by_ref {
            return self.require_compatible_arg_type(expected, actual, arg.span, context);
        }
        match classify_param_binding(expected, actual, arg) {
            ParamBinding::Identity | ParamBinding::Cast(_) | ParamBinding::Const(_) => Ok(()),
            ParamBinding::Callable(target) => {
                let sig = self
                    .resolve_first_class_callable_sig(&target, arg.span, env)
                    .map_err(|err| {
                        Self::param_binding_error(
                            expected,
                            actual,
                            arg,
                            context,
                            err.message.as_str(),
                        )
                    })?;
                self.register_bound_callable_param_sig(owner, sig);
                Ok(())
            }
            ParamBinding::Deprecated(detail)
            | ParamBinding::TypeError(detail)
            | ParamBinding::NeedsRuntimeCheck(detail) => Err(Self::param_binding_error(
                expected, actual, arg, context, &detail,
            )),
            ParamBinding::Rejected => {
                self.require_compatible_arg_type(expected, actual, arg.span, context)
            }
        }
    }

    /// Builds the extended parameter mismatch diagnostic, appending the PHP behaviour that
    /// explains why elephc refuses the binding.
    fn param_binding_error(
        expected: &PhpType,
        actual: &PhpType,
        arg: &Expr,
        context: &str,
        detail: &str,
    ) -> CompileError {
        CompileError::new(
            arg.span,
            &format!(
                "{} expects {:?}, got {:?} — {}",
                context, expected, actual, detail
            ),
        )
    }

    /// Records the signature a bound callable-name string gives a declared `callable`
    /// parameter, so the callee resolves `$f(...)` exactly as it does for a first-class
    /// callable argument.
    fn register_bound_callable_param_sig(
        &mut self,
        owner: Option<(&str, &str)>,
        sig: FunctionSig,
    ) {
        let Some((owner_name, param_name)) = owner else {
            return;
        };
        let key = (owner_name.to_string(), param_name.to_string());
        if self.callable_param_sigs.get(&key) != Some(&sig) {
            self.callable_param_sigs.insert(key, sig);
        }
    }
}
