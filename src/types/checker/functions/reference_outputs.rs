//! Purpose:
//! Carries boxed by-reference storage effects from signature validation to caller inference.
//!
//! Called from:
//! - Function/callable argument validation and expression assignment-effect inference.
//!
//! Key details:
//! - Arguments are already normalized by the shared call planner.
//! - A referenced array element widens its local root to boxed Mixed element storage.
//! - A `null` local bound to a `?ptr<T>` parameter becomes a `?ptr<T>` local.

use crate::parser::ast::{Expr, ExprKind};
use crate::span::Span;
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

impl Checker {
    /// Records property storage before argument effects erase EIR class facts.
    /// Returns true when reference binding itself raises Error, including a nested Mixed read.
    pub(crate) fn record_array_push_reference_place(
        &mut self, receiver: &Expr, env: &TypeEnv,
    ) -> Result<bool, crate::errors::CompileError> {
        match &receiver.kind {
            ExprKind::PropertyAccess { object, property } => {
                let ty = self.infer_type(object, env)?;
                if !matches!(ty.codegen_repr(), PhpType::Object(_)) { return Ok(false); }
                let Some(class) = super::super::single_object_class_name(&ty) else { return Ok(false); };
                let Some(info) = self.classes.get(class.as_str()) else { return Ok(false); };
                let owner = info.property_declaring_classes.get(property)
                    .cloned().unwrap_or_else(|| class.clone());
                let get = crate::names::php_symbol_key(&crate::names::property_hook_get_method(property));
                let set = crate::names::php_symbol_key(&crate::names::property_hook_set_method(property));
                let in_accessor = self.current_class.as_deref() == Some(owner.as_str())
                    && self.current_method.as_ref().is_some_and(|method| method == &get || method == &set);
                let reference_getter = !in_accessor
                    && info.methods.get(&get).is_some_and(|sig| sig.by_ref_return);
                let denied_hook = !in_accessor && !reference_getter
                    && (info.methods.contains_key(&set)
                        || info.methods.get(&get).is_some_and(|sig| !sig.by_ref_return));
                let setter = info.property_set_visibilities.get(property)
                    .or_else(|| info.property_visibilities.get(property))
                    .cloned().unwrap_or(crate::parser::ast::Visibility::Public);
                let rejection = if denied_hook {
                    Some(format!("Indirect modification of {owner}::${property} is not allowed"))
                } else if info.readonly_properties.contains(property) {
                    Some(format!("Cannot indirectly modify readonly property {owner}::${property}"))
                } else if !reference_getter && !self.can_access_member(&owner, &setter) {
                    let scope = self.current_class.as_ref().map_or_else(
                        || "global scope".to_string(), |class| format!("scope {class}"));
                    Some(format!("Cannot indirectly modify {}(set) property {owner}::${property} from {scope}",
                        Self::visibility_label(&setter)))
                } else { None };
                let plan = crate::types::ArrayPushPropertyPlan { class_name: class.clone(), rejection };
                let key = (self.current_loop_storage_scope.clone(), receiver.span);
                if self.array_push_property_plans.get(&key).is_some_and(|existing| existing != &plan) {
                    return Err(crate::errors::CompileError::new(receiver.span,
                        "array_push() property reference has ambiguous receiver classes at this source position"));
                }
                let permitted = plan.rejection.is_none();
                self.array_push_property_plans.insert(key, plan);
                if permitted && self.ref_property_slot_addressability(object, property, env)?
                    .is_some_and(|(_, addressable)| addressable)
                {
                    self.reference_property_promotions.insert((class, property.clone()));
                }
                return Ok(!permitted);
            }
            ExprKind::ArrayAccess { array, .. }
            | ExprKind::NamedArg { value: array, .. }
            | ExprKind::ErrorSuppress(array) => return self.record_array_push_reference_place(array, env),
            _ => {},
        }
        Ok(false)
    }

    /// Records a local whose reference binding requires canonical boxed storage after the call.
    ///
    /// A `null` local bound to a `?ptr<T>` parameter leaves the call as that `?ptr<T>`: the
    /// callee may store a live address through it, and lowering gives the slot that pointer word.
    pub(crate) fn record_boxed_reference_output(
        &mut self,
        arg: &Expr,
        expected: &PhpType,
        actual: &PhpType,
        call_span: Span,
        env: &TypeEnv,
    ) {
        let output_ty = if expected.is_php_array()
            && matches!(actual, PhpType::Array(_) | PhpType::AssocArray { .. })
        {
            PhpType::php_array()
        } else if (expected.codegen_repr() == PhpType::Mixed
            || matches!(expected, PhpType::Object(_)))
            && actual.codegen_repr() != PhpType::Mixed
        {
            PhpType::Mixed
        } else if crate::types::is_nullable_pointer_type(expected)
            && matches!(actual.codegen_repr(), PhpType::Void | PhpType::Never)
        {
            expected.clone()
        } else {
            return;
        };
        if !call_span.identifies_a_node() {
            return;
        }
        let mut arg = arg;
        while let ExprKind::NamedArg { value, .. } | ExprKind::ErrorSuppress(value) = &arg.kind {
            arg = value;
        }
        let (name, output_ty) = match &arg.kind {
            ExprKind::Variable(name) => {
                if output_ty == PhpType::Mixed && arg.span.identifies_a_node() {
                    self.boxed_reference_promotion_sites
                        .entry((self.current_loop_storage_scope.clone(), arg.span))
                        .or_default()
                        .insert(name.clone());
                }
                (name, output_ty)
            }
            ExprKind::ArrayAccess { array, .. } if output_ty == PhpType::Mixed => {
                let mut root = array.as_ref();
                while let ExprKind::ArrayAccess { array, .. } = &root.kind {
                    root = array;
                }
                let ExprKind::Variable(name) = &root.kind else {
                    return;
                };
                let Some(root_ty) = env.get(name) else {
                    return;
                };
                let widened = match root_ty.codegen_repr() {
                    PhpType::Array(_) => PhpType::Array(Box::new(PhpType::Mixed)),
                    PhpType::AssocArray { key, .. } => PhpType::AssocArray {
                        key,
                        value: Box::new(PhpType::Mixed),
                    },
                    _ => return,
                };
                (name, widened)
            }
            _ => return,
        };
        self.boxed_reference_outputs
            .entry((self.current_loop_storage_scope.clone(), call_span))
            .or_default()
            .insert(name.clone(), output_ty);
    }

    /// Returns whether this call has already approved boxing the same local for an earlier slot.
    pub(crate) fn boxed_reference_promotion_pending(
        &self,
        arg: &Expr,
        call_span: Span,
    ) -> bool {
        let mut arg = arg;
        while let ExprKind::NamedArg { value, .. } | ExprKind::ErrorSuppress(value) = &arg.kind {
            arg = value;
        }
        let ExprKind::Variable(name) = &arg.kind else {
            return false;
        };
        self.boxed_reference_outputs
            .get(&(self.current_loop_storage_scope.clone(), call_span))
            .and_then(|outputs| outputs.get(name))
            .is_some_and(|ty| ty.codegen_repr() == PhpType::Mixed)
    }

    /// Applies one successful call's storage changes before checking subsequent expressions.
    pub(crate) fn apply_boxed_reference_outputs(&mut self, span: Span, env: &mut TypeEnv) {
        let key = (self.current_loop_storage_scope.clone(), span);
        if let Some(outputs) = self.boxed_reference_outputs.remove(&key) {
            for (name, output_ty) in outputs {
                env.insert(name, output_ty);
            }
        }
    }
}
