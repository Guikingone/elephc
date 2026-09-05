//! Purpose:
//! Lowers a generator function body into the flat step list the generator frame executes.
//!
//! Called from:
//! - `super::eval_generator_new()` once, when a generator object is created.
//!
//! Key details:
//! - ONLY constructs that contain a `yield` are flattened. Any statement that does not is kept
//!   whole and handed to `execute_statements()`, so the ordinary evaluator still does all the
//!   real work and this file owns nothing but the control flow around suspension points.
//! - The step list is a program with an explicit counter, which is what makes resuming free:
//!   the frame stores the index of the next step and continues there. A resumable tree walker
//!   would have to reify every loop's and every `try`'s position anyway, and would then have to
//!   re-descend the tree on each resume.

use super::*;
use crate::context::{EvalGeneratorProgram, EvalGeneratorStep};
use crate::parser::{EVAL_YIELD_FROM_INTRINSIC, EVAL_YIELD_INTRINSIC};

/// The reserved scope name a `return yield ...;` stores its sent value under.
///
/// PHP allows the sent value to become the return value, so the yield needs somewhere to put it
/// before the return reads it back. The name cannot collide with a PHP variable because `$` is
/// not part of it and PHP identifiers cannot contain a NUL.
pub(super) const EVAL_GENERATOR_SENT_SLOT: &str = "\0generator\0sent";

/// Lowers one function body, or refuses a yield this lowering does not model.
pub(super) fn lower_generator_body(
    body: &[EvalStmt],
) -> Result<EvalGeneratorProgram, EvalStatus> {
    let mut lowering = Lowering {
        steps: Vec::new(),
        loops: Vec::new(),
        foreach_slots: 0,
    };
    lowering.lower_statements(body)?;
    lowering.steps.push(EvalGeneratorStep::Return(None));
    Ok(EvalGeneratorProgram {
        steps: lowering.steps,
        foreach_slots: lowering.foreach_slots,
    })
}

/// Where a `break` and a `continue` jump to inside one lowered loop.
struct LoopLabels {
    continue_patches: Vec<usize>,
    break_patches: Vec<usize>,
}

/// Accumulates the step list while walking the body.
struct Lowering {
    steps: Vec<EvalGeneratorStep>,
    loops: Vec<LoopLabels>,
    foreach_slots: usize,
}

impl Lowering {
    /// Lowers a statement list, batching yield-free runs into single steps.
    fn lower_statements(&mut self, body: &[EvalStmt]) -> Result<(), EvalStatus> {
        let mut pending: Vec<EvalStmt> = Vec::new();
        for statement in body {
            if statement_needs_lowering(statement) {
                self.flush(&mut pending);
                self.lower_statement(statement)?;
            } else {
                pending.push(statement.clone());
            }
        }
        self.flush(&mut pending);
        Ok(())
    }

    /// Emits the batched yield-free statements as one atomic step.
    fn flush(&mut self, pending: &mut Vec<EvalStmt>) {
        if !pending.is_empty() {
            self.steps
                .push(EvalGeneratorStep::Run(std::mem::take(pending)));
        }
    }

    /// Lowers one statement that contains a yield, a break, or a continue.
    fn lower_statement(&mut self, statement: &EvalStmt) -> Result<(), EvalStatus> {
        match statement {
            EvalStmt::Expr(expr) => self.lower_yield_expr(expr, None),
            EvalStmt::StoreVar { name, value } => self.lower_yield_expr(value, Some(name.clone())),
            EvalStmt::Return(Some(expr)) if expression_contains_yield(expr) => {
                let slot = EVAL_GENERATOR_SENT_SLOT.to_string();
                self.lower_yield_expr(expr, Some(slot.clone()))?;
                self.steps
                    .push(EvalGeneratorStep::Return(Some(EvalExpr::LoadVar(slot))));
                Ok(())
            }
            EvalStmt::Return(value) => {
                self.steps.push(EvalGeneratorStep::Return(value.clone()));
                Ok(())
            }
            EvalStmt::Break(level) => self.lower_loop_exit(*level, true),
            EvalStmt::Continue(level) => self.lower_loop_exit(*level, false),
            EvalStmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(condition, then_branch, else_branch),
            EvalStmt::While { condition, body } => self.lower_while(condition, body),
            EvalStmt::DoWhile { body, condition } => self.lower_do_while(body, condition),
            EvalStmt::For {
                init,
                condition,
                update,
                body,
            } => self.lower_for(init, condition.as_ref(), update, body),
            EvalStmt::Foreach {
                array,
                key_name,
                value_name,
                value_by_ref,
                body,
            } => self.lower_foreach(array, key_name.as_deref(), value_name, *value_by_ref, body),
            // A `yield` inside `try`/`switch` needs the frame to model handler and arm state,
            // which this lowering does not do yet. Refusing keeps a half-modelled suspension
            // from silently skipping a `finally`.
            _ => Err(EvalStatus::UnsupportedConstruct),
        }
    }

    /// Lowers one expression that carries a yield in a position this lowering models.
    ///
    /// The three positions are the ones PHP programs actually use: a bare `yield ...;`, the
    /// right-hand side of an assignment to a variable, and the operand of a `return`. A yield
    /// buried deeper in an expression would need the evaluator itself to suspend mid-expression.
    fn lower_yield_expr(
        &mut self,
        expr: &EvalExpr,
        into: Option<String>,
    ) -> Result<(), EvalStatus> {
        match expr {
            EvalExpr::Call { name, args } if name == EVAL_YIELD_INTRINSIC => {
                let (key, value) = match args.as_slice() {
                    [value] => (None, value.value().clone()),
                    [key, value] => (Some(key.value().clone()), value.value().clone()),
                    _ => return Err(EvalStatus::RuntimeFatal),
                };
                self.steps
                    .push(EvalGeneratorStep::Yield { key, value, into });
                Ok(())
            }
            EvalExpr::Call { name, args } if name == EVAL_YIELD_FROM_INTRINSIC => {
                let [source] = args.as_slice() else {
                    return Err(EvalStatus::RuntimeFatal);
                };
                self.steps.push(EvalGeneratorStep::YieldFrom {
                    source: source.value().clone(),
                    into,
                });
                Ok(())
            }
            _ => Err(EvalStatus::UnsupportedConstruct),
        }
    }

    /// Emits the jump for a `break` or a `continue` targeting an enclosing lowered loop.
    fn lower_loop_exit(&mut self, level: u32, is_break: bool) -> Result<(), EvalStatus> {
        let depth = self.loops.len();
        let level = level.max(1) as usize;
        if level > depth {
            return Err(EvalStatus::UnsupportedConstruct);
        }
        let index = depth - level;
        let patch = self.steps.len();
        self.steps.push(EvalGeneratorStep::Jump(usize::MAX));
        if is_break {
            self.loops[index].break_patches.push(patch);
        } else {
            self.loops[index].continue_patches.push(patch);
        }
        Ok(())
    }

    /// Lowers `if`/`else` into a conditional jump around each branch.
    fn lower_if(
        &mut self,
        condition: &EvalExpr,
        then_branch: &[EvalStmt],
        else_branch: &[EvalStmt],
    ) -> Result<(), EvalStatus> {
        let branch_patch = self.steps.len();
        self.steps.push(EvalGeneratorStep::JumpIfFalse {
            condition: condition.clone(),
            target: usize::MAX,
        });
        self.lower_statements(then_branch)?;
        if else_branch.is_empty() {
            let after = self.steps.len();
            self.patch_jump(branch_patch, after);
            return Ok(());
        }
        let skip_else = self.steps.len();
        self.steps.push(EvalGeneratorStep::Jump(usize::MAX));
        let else_start = self.steps.len();
        self.patch_jump(branch_patch, else_start);
        self.lower_statements(else_branch)?;
        let after = self.steps.len();
        self.patch_jump(skip_else, after);
        Ok(())
    }

    /// Lowers `while` into a test at the top and a back edge at the bottom.
    fn lower_while(
        &mut self,
        condition: &EvalExpr,
        body: &[EvalStmt],
    ) -> Result<(), EvalStatus> {
        let top = self.steps.len();
        let exit_patch = self.steps.len();
        self.steps.push(EvalGeneratorStep::JumpIfFalse {
            condition: condition.clone(),
            target: usize::MAX,
        });
        self.loops.push(LoopLabels {
            continue_patches: Vec::new(),
            break_patches: Vec::new(),
        });
        self.lower_statements(body)?;
        self.steps.push(EvalGeneratorStep::Jump(top));
        let after = self.steps.len();
        self.patch_jump(exit_patch, after);
        self.close_loop(top, after);
        Ok(())
    }

    /// Lowers `do`/`while`, whose body runs before the first test.
    fn lower_do_while(
        &mut self,
        body: &[EvalStmt],
        condition: &EvalExpr,
    ) -> Result<(), EvalStatus> {
        let top = self.steps.len();
        self.loops.push(LoopLabels {
            continue_patches: Vec::new(),
            break_patches: Vec::new(),
        });
        self.lower_statements(body)?;
        let test = self.steps.len();
        self.steps.push(EvalGeneratorStep::JumpIfFalse {
            condition: condition.clone(),
            target: usize::MAX,
        });
        self.steps.push(EvalGeneratorStep::Jump(top));
        let after = self.steps.len();
        self.patch_jump(test, after);
        self.close_loop(test, after);
        Ok(())
    }

    /// Lowers `for`, whose update runs on the continue edge as well as the fall-through.
    fn lower_for(
        &mut self,
        init: &[EvalStmt],
        condition: Option<&EvalExpr>,
        update: &[EvalStmt],
        body: &[EvalStmt],
    ) -> Result<(), EvalStatus> {
        if !init.is_empty() {
            self.steps.push(EvalGeneratorStep::Run(init.to_vec()));
        }
        let top = self.steps.len();
        let exit_patch = condition.map(|condition| {
            let patch = self.steps.len();
            self.steps.push(EvalGeneratorStep::JumpIfFalse {
                condition: condition.clone(),
                target: usize::MAX,
            });
            patch
        });
        self.loops.push(LoopLabels {
            continue_patches: Vec::new(),
            break_patches: Vec::new(),
        });
        self.lower_statements(body)?;
        let continue_target = self.steps.len();
        if !update.is_empty() {
            self.steps.push(EvalGeneratorStep::Run(update.to_vec()));
        }
        self.steps.push(EvalGeneratorStep::Jump(top));
        let after = self.steps.len();
        if let Some(exit_patch) = exit_patch {
            self.patch_jump(exit_patch, after);
        }
        self.close_loop(continue_target, after);
        Ok(())
    }

    /// Lowers `foreach` into a frame-held iteration slot plus a bind-or-exit step.
    fn lower_foreach(
        &mut self,
        subject: &EvalExpr,
        key_name: Option<&str>,
        value_name: &str,
        value_by_ref: bool,
        body: &[EvalStmt],
    ) -> Result<(), EvalStatus> {
        if value_by_ref {
            // A by-reference foreach whose body suspends would have to keep the alias alive
            // across the suspension; refusing is better than writing through a stale binding.
            return Err(EvalStatus::UnsupportedConstruct);
        }
        let slot = self.foreach_slots;
        self.foreach_slots += 1;
        self.steps.push(EvalGeneratorStep::ForeachInit {
            subject: subject.clone(),
            slot,
        });
        let top = self.steps.len();
        let next_patch = self.steps.len();
        self.steps.push(EvalGeneratorStep::ForeachNext {
            slot,
            key_name: key_name.map(str::to_string),
            value_name: value_name.to_string(),
            exit: usize::MAX,
        });
        self.loops.push(LoopLabels {
            continue_patches: Vec::new(),
            break_patches: Vec::new(),
        });
        self.lower_statements(body)?;
        self.steps.push(EvalGeneratorStep::Jump(top));
        let after = self.steps.len();
        if let Some(EvalGeneratorStep::ForeachNext { exit, .. }) = self.steps.get_mut(next_patch) {
            *exit = after;
        }
        self.close_loop(top, after);
        Ok(())
    }

    /// Resolves the pending `break` and `continue` jumps of the innermost lowered loop.
    fn close_loop(&mut self, continue_target: usize, break_target: usize) {
        let Some(labels) = self.loops.pop() else {
            return;
        };
        for patch in labels.continue_patches {
            self.patch_jump(patch, continue_target);
        }
        for patch in labels.break_patches {
            self.patch_jump(patch, break_target);
        }
    }

    /// Rewrites one placeholder jump target now that the destination index is known.
    fn patch_jump(&mut self, patch: usize, target: usize) {
        match self.steps.get_mut(patch) {
            Some(EvalGeneratorStep::Jump(slot)) => *slot = target,
            Some(EvalGeneratorStep::JumpIfFalse { target: slot, .. }) => *slot = target,
            _ => {}
        }
    }
}

/// Returns whether a statement has to be lowered rather than run atomically.
///
/// A statement is lowered when it contains a suspension point, or when it is a `break` or a
/// `continue` that would otherwise escape the atomic chunk it was batched into.
fn statement_needs_lowering(statement: &EvalStmt) -> bool {
    matches!(statement, EvalStmt::Break(_) | EvalStmt::Continue(_))
        || statement_contains_yield(statement)
}

/// Returns whether a statement contains a yield anywhere inside it.
pub(super) fn statement_contains_yield(statement: &EvalStmt) -> bool {
    match statement {
        EvalStmt::Expr(expr) | EvalStmt::Echo(expr) => expression_contains_yield(expr),
        EvalStmt::StoreVar { value, .. } => expression_contains_yield(value),
        EvalStmt::Return(value) => value.as_ref().is_some_and(expression_contains_yield),
        EvalStmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expression_contains_yield(condition)
                || then_branch.iter().any(statement_contains_yield)
                || else_branch.iter().any(statement_contains_yield)
        }
        EvalStmt::While { condition, body } => {
            expression_contains_yield(condition) || body.iter().any(statement_contains_yield)
        }
        EvalStmt::DoWhile { body, condition } => {
            expression_contains_yield(condition) || body.iter().any(statement_contains_yield)
        }
        EvalStmt::For {
            init,
            condition,
            update,
            body,
        } => {
            init.iter().any(statement_contains_yield)
                || condition.as_ref().is_some_and(expression_contains_yield)
                || update.iter().any(statement_contains_yield)
                || body.iter().any(statement_contains_yield)
        }
        EvalStmt::Foreach { array, body, .. } => {
            expression_contains_yield(array) || body.iter().any(statement_contains_yield)
        }
        EvalStmt::Switch { expr, cases } => {
            expression_contains_yield(expr)
                || cases.iter().any(|case| {
                    case.condition.as_ref().is_some_and(expression_contains_yield)
                        || case.body.iter().any(statement_contains_yield)
                })
        }
        EvalStmt::Try {
            body,
            catches,
            finally_body,
        } => {
            body.iter().any(statement_contains_yield)
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(statement_contains_yield))
                || finally_body.iter().any(statement_contains_yield)
        }
        _ => false,
    }
}

/// Returns whether an expression contains a yield marker anywhere inside it.
///
/// Only the shallow shapes need to be exact: a yield the lowering cannot place is refused, and
/// this predicate exists to decide whether a statement may be run atomically at all. Missing a
/// deeply buried yield would run it through the ordinary evaluator, where the marker resolves
/// to no function and fails loudly rather than silently.
pub(super) fn expression_contains_yield(expr: &EvalExpr) -> bool {
    match expr {
        EvalExpr::Call { name, args } => {
            name == EVAL_YIELD_INTRINSIC
                || name == EVAL_YIELD_FROM_INTRINSIC
                || args.iter().any(|arg| expression_contains_yield(arg.value()))
        }
        EvalExpr::Assign { value, .. } => expression_contains_yield(value),
        EvalExpr::Binary { left, right, .. } => {
            expression_contains_yield(left) || expression_contains_yield(right)
        }
        EvalExpr::Unary { expr, .. } => expression_contains_yield(expr),
        _ => false,
    }
}

/// Returns whether a function body makes its function a generator.
pub(in crate::interpreter) fn eval_body_is_generator(body: &[EvalStmt]) -> bool {
    body.iter().any(statement_contains_yield)
}
