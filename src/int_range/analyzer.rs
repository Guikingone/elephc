//! Purpose:
//! Abstract interpreter of the integer-range analysis. Walks every executable body of the
//! program with interval facts for local variables and records, for each `+`, `-`, `*`, `++`
//! and `--` site, whether the integer form of that operation can overflow.
//!
//! Called from:
//! - `crate::int_range::analyze_program()`.
//!
//! Key details:
//! - A site is recorded "safe" only when both operands are definite ints whose intervals keep
//!   the result inside `i64`. Every visit of a site is recorded; one unsafe visit wins.
//! - Loops run to a fixpoint with widening while recording is off, then one recording pass runs
//!   from the stable loop-head state, so verdicts always come from a sound invariant.
//! - Conditions narrow intervals (`$i < $n`, `$n > 2`, ...); exceptions are approximated by
//!   forgetting every variable written in the `try` body when entering a `catch`.
//! - When two sibling operands write and read the same variable, that variable is forgotten
//!   before evaluating them, so verdicts never depend on operand evaluation order.

use std::collections::{HashMap, HashSet};

use crate::parser::ast::{BinOp, CastType, ClassMethod, Expr, ExprKind, Stmt, StmtKind, TypeExpr};

use super::domain::{join_opt, join_states, narrow, ops, widen_states, AbsVal, Env, State, MAX, MIN};
use super::scan::{
    collect_reads_expr, collect_writes_expr, collect_writes_stmts, global_alias_target, root_variable,
    scan_body, symbol_key, walk_stmts, Node, Param, ProgramInfo,
};
use super::{ArithKind, Facts};

/// Upper bound on any array element count or string length: user address spaces on every
/// supported target are at most 57 bits wide, and each element or byte needs storage.
const MEMORY_BOUND: i64 = 1 << 57;

/// Loop fixpoint iterations after which the head state is widened.
const WIDEN_AFTER: usize = 2;

/// Descending iterations run after the widened fixpoint is reached.
const NARROWING_PASSES: usize = 2;

/// Hard cap on fixpoint iterations (widening normally converges far earlier).
const MAX_ITERATIONS: usize = 64;

/// Break/continue accumulators for one enclosing loop or switch.
#[derive(Default)]
struct Frame {
    is_switch: bool,
    breaks: State,
    continues: State,
}

/// Interprocedural parameter ranges: the input of one analysis round and the call-site
/// observations it produces for the next one.
#[derive(Debug, Default)]
pub(crate) struct CallSites {
    /// Parameter ranges assumed at function entry this round.
    pub(crate) assumed: HashMap<String, Vec<AbsVal>>,
    /// When `true`, a tracked function missing from `assumed` has no reachable direct call yet
    /// and its body is analyzed as unreachable (the optimistic start of the fixpoint); when
    /// `false` its parameters are simply unknown.
    pub(crate) optimistic: bool,
    /// Join of the argument values seen at reachable direct calls (`None`: position not seen).
    pub(crate) observed: HashMap<String, Vec<Option<AbsVal>>>,
}

/// Interpreter state for one body (function, method, closure, or the top-level script).
pub(crate) struct Analyzer<'a> {
    info: &'a ProgramInfo,
    facts: &'a mut Facts,
    calls: &'a mut CallSites,
    recording: bool,
    pinned: HashSet<String>,
    all_pinned: bool,
    frames: Vec<Frame>,
    thresholds: Vec<i64>,
}

impl<'a> Analyzer<'a> {
    /// Creates an analyzer for a body with the given pinned variables.
    fn new(
        info: &'a ProgramInfo,
        facts: &'a mut Facts,
        calls: &'a mut CallSites,
        body: &[Stmt],
        pinned: HashSet<String>,
        all_pinned: bool,
    ) -> Self {
        Self {
            info,
            facts,
            calls,
            recording: true,
            pinned,
            all_pinned,
            frames: Vec::new(),
            thresholds: widening_thresholds(body),
        }
    }

    /// Analyzes the top-level script (and, transitively, every declaration it contains).
    pub(crate) fn analyze_script(
        info: &'a ProgramInfo,
        facts: &'a mut Facts,
        calls: &'a mut CallSites,
        program: &[Stmt],
        web: bool,
    ) {
        let scan = scan_body(program, &[], info);
        let mut pinned = scan.pinned;
        pinned.extend(info.top_level_pinned.iter().cloned());
        let mut env = Env::default();
        if !web && !pinned.contains("argc") {
            // The kernel caps argv at `MAX_ARG_STRINGS` (0x7FFFFFFF) entries.
            env.set("argc", AbsVal::Int(0, i32::MAX as i64));
        }
        let mut analyzer = Analyzer::new(info, facts, calls, program, pinned, scan.unanalyzable);
        analyzer.exec(program, Some(env));
    }

    /// Analyzes a function-like body in a fresh scope.
    fn analyze_function(
        &mut self,
        function: Option<&str>,
        params: &[Param],
        body: &[Stmt],
        extra_pins: &[String],
    ) {
        for param in params {
            if let Some(default) = &param.2 {
                self.eval_detached(default);
            }
        }
        let scan = scan_body(body, params, self.info);
        let mut pinned = scan.pinned;
        pinned.extend(extra_pins.iter().cloned());
        let mut env = Env::default();
        let tracked = function.filter(|name| self.info.tracks_call_sites(name));
        let assumed = tracked.and_then(|name| self.calls.assumed.get(&symbol_key(name)).cloned());
        let unreachable = tracked.is_some() && assumed.is_none() && self.calls.optimistic;
        for (index, (name, type_expr, default, by_ref)) in params.iter().enumerate() {
            if *by_ref {
                continue;
            }
            let null_default = matches!(default.as_ref().map(|d| &d.kind), Some(ExprKind::Null));
            let typed_int = !null_default && matches!(type_expr, Some(TypeExpr::Int));
            let observed = assumed.as_ref().and_then(|ranges| ranges.get(index).copied());
            let value = match (typed_int, observed) {
                // Every call passed a definite int: the parameter holds exactly those values.
                (_, Some(AbsVal::Int(lo, hi))) => AbsVal::Int(lo, hi),
                // An `int` parameter is an int whatever the caller passed (coercion or TypeError).
                (true, _) => AbsVal::FULL_INT,
                (false, _) => AbsVal::Top,
            };
            env.set(name, value);
        }
        let mut child = Analyzer::new(
            self.info,
            &mut *self.facts,
            &mut *self.calls,
            body,
            pinned,
            scan.unanalyzable,
        );
        child.exec(body, if unreachable { None } else { Some(env) });
    }

    /// Analyzes a class-like member list.
    fn analyze_members(&mut self, methods: &[ClassMethod], constants: &[crate::parser::ast::ClassConst]) {
        for method in methods {
            self.analyze_function(None, &method.params, &method.body, &[]);
        }
        for constant in constants {
            self.eval_detached(&constant.value);
        }
    }

    /// Evaluates a context-free expression (defaults, constants) in a state with no facts.
    fn eval_detached(&mut self, expr: &Expr) {
        if !self.recording {
            return;
        }
        let mut state = Some(Env::default());
        let saved = std::mem::replace(&mut self.all_pinned, true);
        self.eval(expr, &mut state);
        self.all_pinned = saved;
    }

    // ----- variables -----------------------------------------------------------------------

    /// Returns `true` when a variable may change behind the analysis' back.
    fn is_pinned(&self, name: &str) -> bool {
        self.all_pinned || name == "this" || global_alias_target(name).is_some() || self.pinned.contains(name)
    }

    /// Reads the abstract value of a local.
    fn read(&self, state: &State, name: &str) -> AbsVal {
        if self.is_pinned(name) {
            return AbsVal::Top;
        }
        state.as_ref().map_or(AbsVal::Top, |env| env.get(name))
    }

    /// Writes the abstract value of a local.
    fn write(&self, state: &mut State, name: &str, value: AbsVal) {
        if let Some(env) = state {
            let value = if self.is_pinned(name) { AbsVal::Top } else { value };
            env.set(name, value);
        }
    }

    /// Forgets every listed variable.
    fn forget<'n>(&self, state: &mut State, names: impl IntoIterator<Item = &'n String>) {
        if let Some(env) = state {
            for name in names {
                env.set(name, AbsVal::Top);
            }
        }
    }

    /// Records one arithmetic verdict.
    fn record(&mut self, state: &State, expr: &Expr, kind: ArithKind, safe: bool) {
        if !self.recording || state.is_none() || expr.span.line == 0 {
            return;
        }
        let verdict = self.facts.entry((expr.span, kind)).or_insert(true);
        *verdict &= safe;
    }

    // ----- statements ----------------------------------------------------------------------

    /// Executes statements in order. Unreachable statements are still visited so nested
    /// declarations are analyzed and their sites recorded.
    fn exec(&mut self, stmts: &[Stmt], mut state: State) -> State {
        for stmt in stmts {
            state = self.stmt(stmt, state);
        }
        state
    }

    /// Executes one statement.
    fn stmt(&mut self, stmt: &Stmt, mut state: State) -> State {
        match &stmt.kind {
            StmtKind::Echo(value) | StmtKind::ExprStmt(value) | StmtKind::ConstDecl { value, .. } => {
                self.eval(value, &mut state);
                state
            }
            StmtKind::Assign { name, value } => {
                let result = self.eval(value, &mut state);
                self.write(&mut state, name, result);
                state
            }
            StmtKind::TypedAssign { type_expr, name, value } => {
                let result = self.eval(value, &mut state);
                let result = match (type_expr, result) {
                    (TypeExpr::Int, AbsVal::Int(..)) => result,
                    _ => AbsVal::Top,
                };
                self.write(&mut state, name, result);
                state
            }
            StmtKind::RefAssign { target, source } => {
                self.eval(source, &mut state);
                self.write(&mut state, target, AbsVal::Top);
                state
            }
            StmtKind::RefAssignToTarget { target, source, .. } => {
                self.eval_operands(&[target, source], &mut state);
                self.forget_root(&mut state, target);
                state
            }
            StmtKind::If {
                condition,
                then_body,
                elseif_clauses,
                else_body,
            } => {
                let (then_state, mut rest) = self.cond(condition, state);
                let mut out = self.exec(then_body, then_state);
                for (condition, body) in elseif_clauses {
                    let (branch, next) = self.cond(condition, rest);
                    let end = self.exec(body, branch);
                    out = join_states(&out, &end);
                    rest = next;
                }
                let end = match else_body {
                    Some(body) => self.exec(body, rest),
                    None => rest,
                };
                join_states(&out, &end)
            }
            StmtKind::IfDef { then_body, else_body, .. } => {
                let then_end = self.exec(then_body, state.clone());
                let else_end = match else_body {
                    Some(body) => self.exec(body, state),
                    None => state,
                };
                join_states(&then_end, &else_end)
            }
            StmtKind::While { condition, body } => self.run_loop(state, false, &mut |a, head| {
                let (inside, outside) = a.cond(condition, head);
                let end = a.exec(body, inside);
                let frame = a.frames.pop().expect("loop frame");
                let back = join_states(&end, &frame.continues);
                (back, join_states(&outside, &frame.breaks))
            }),
            StmtKind::DoWhile { body, condition } => self.run_loop(state, false, &mut |a, head| {
                let end = a.exec(body, head);
                let frame = a.frames.pop().expect("loop frame");
                let before_condition = join_states(&end, &frame.continues);
                let (again, outside) = a.cond(condition, before_condition);
                (again, join_states(&outside, &frame.breaks))
            }),
            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init) = init {
                    state = self.stmt(init, state);
                }
                self.run_loop(state, false, &mut |a, head| {
                    let (inside, outside) = match condition {
                        Some(condition) => a.cond(condition, head),
                        None => (head, None),
                    };
                    let end = a.exec(body, inside);
                    let frame = a.frames.pop().expect("loop frame");
                    let mut back = join_states(&end, &frame.continues);
                    if let Some(update) = update {
                        back = a.stmt(update, back);
                    }
                    (back, join_states(&outside, &frame.breaks))
                })
            }
            StmtKind::Foreach {
                array,
                key_var,
                value_var,
                body,
                ..
            } => {
                self.eval(array, &mut state);
                self.run_loop(state, false, &mut |a, head| {
                    let mut inside = head.clone();
                    if let Some(key_var) = key_var {
                        a.write(&mut inside, key_var, AbsVal::Top);
                    }
                    a.write(&mut inside, value_var, AbsVal::Top);
                    let end = a.exec(body, inside);
                    let frame = a.frames.pop().expect("loop frame");
                    let back = join_states(&end, &frame.continues);
                    (back, join_states(&head, &frame.breaks))
                })
            }
            StmtKind::Switch {
                subject,
                cases,
                default,
            } => {
                self.eval(subject, &mut state);
                // Patterns run in order until one matches; any prefix may be the matching one.
                let mut matched = None;
                for (patterns, _) in cases {
                    for pattern in patterns {
                        self.eval(pattern, &mut state);
                    }
                    matched = join_states(&matched, &state);
                }
                let unmatched = if default.is_some() { None } else { state.clone() };
                let entry = join_states(&matched, &state);
                // Fallthrough and `default` placement are over-approximated: every body may be
                // entered from the switch head or from the end of any body.
                self.run_loop(entry, true, &mut |a, head| {
                    let mut ends = None;
                    for (_, body) in cases {
                        let end = a.exec(body, head.clone());
                        ends = join_states(&ends, &end);
                    }
                    if let Some(body) = default {
                        let end = a.exec(body, head.clone());
                        ends = join_states(&ends, &end);
                    }
                    let frame = a.frames.pop().expect("switch frame");
                    let exits = join_states(&join_states(&ends, &frame.breaks), &frame.continues);
                    (ends, join_states(&exits, &unmatched))
                })
            }
            StmtKind::Include { path, .. } => {
                self.eval(path, &mut state);
                // An unresolved include may write any local (the body is marked unanalyzable).
                state.map(|_| Env::default())
            }
            StmtKind::IncludeOnceGuard { body, .. } => {
                let end = self.exec(body, state.clone());
                join_states(&state, &end)
            }
            StmtKind::Throw(value) => {
                self.eval(value, &mut state);
                None
            }
            StmtKind::Synthetic(body) | StmtKind::NamespaceBlock { body, .. } => self.exec(body, state),
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => self.exec_try(try_body, catches, finally_body.as_deref(), state),
            StmtKind::Break(levels) => {
                self.jump(&state, *levels, false);
                None
            }
            StmtKind::Continue(levels) => {
                self.jump(&state, *levels, true);
                None
            }
            StmtKind::Return(value) => {
                if let Some(value) = value {
                    self.eval(value, &mut state);
                }
                None
            }
            StmtKind::ListUnpack { vars, value } => {
                self.eval(value, &mut state);
                for var in vars {
                    self.write(&mut state, var, AbsVal::Top);
                }
                state
            }
            StmtKind::Global { vars } => {
                for var in vars {
                    self.write(&mut state, var, AbsVal::Top);
                }
                state
            }
            StmtKind::StaticVar { name, init } => {
                self.eval_detached(init);
                self.write(&mut state, name, AbsVal::Top);
                state
            }
            StmtKind::ArrayAssign { array, index, value } => {
                self.eval_operands(&[index, value], &mut state);
                self.write(&mut state, array, AbsVal::Top);
                state
            }
            StmtKind::NestedArrayAssign { target, value } => {
                self.eval_operands(&[target, value], &mut state);
                self.forget_root(&mut state, target);
                state
            }
            StmtKind::ArrayPush { array, value } => {
                self.eval(value, &mut state);
                self.write(&mut state, array, AbsVal::Top);
                state
            }
            StmtKind::PropertyAssign { object, value, .. }
            | StmtKind::PropertyArrayPush { object, value, .. } => {
                self.eval_operands(&[object, value], &mut state);
                state
            }
            StmtKind::PropertyArrayAssign {
                object, index, value, ..
            } => {
                self.eval_operands(&[object, index, value], &mut state);
                state
            }
            StmtKind::StaticPropertyAssign { value, .. } | StmtKind::StaticPropertyArrayPush { value, .. } => {
                self.eval(value, &mut state);
                state
            }
            StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
                self.eval_operands(&[index, value], &mut state);
                state
            }
            StmtKind::DynamicStaticPropertyWrite {
                property, index, value, ..
            } => {
                let mut operands: Vec<&Expr> = vec![property];
                if let Some(index) = index {
                    operands.push(index);
                }
                operands.push(value);
                self.eval_operands(&operands, &mut state);
                state
            }
            StmtKind::FunctionDecl { name, params, body, .. } => {
                if self.recording {
                    self.analyze_function(Some(name), params, body, &[]);
                }
                state
            }
            StmtKind::ClassDecl {
                properties,
                methods,
                constants,
                ..
            }
            | StmtKind::InterfaceDecl {
                properties,
                methods,
                constants,
                ..
            }
            | StmtKind::TraitDecl {
                properties,
                methods,
                constants,
                ..
            } => {
                if self.recording {
                    for property in properties {
                        if let Some(default) = &property.default {
                            self.eval_detached(default);
                        }
                    }
                    self.analyze_members(methods, constants);
                }
                state
            }
            StmtKind::EnumDecl {
                cases,
                methods,
                constants,
                ..
            } => {
                if self.recording {
                    for case in cases {
                        if let Some(value) = &case.value {
                            self.eval_detached(value);
                        }
                    }
                    self.analyze_members(methods, constants);
                }
                state
            }
            StmtKind::Goto(_) | StmtKind::Label(_) => {
                // Bodies containing labels are scanned as unanalyzable (`all_pinned`), so no
                // fact survives across the jump anyway.
                state.map(|_| Env::default())
            }
            StmtKind::IncludeOnceMark { .. }
            | StmtKind::NamespaceDecl { .. }
            | StmtKind::UseDecl { .. }
            | StmtKind::FunctionVariantGroup { .. }
            | StmtKind::FunctionVariantMark { .. }
            | StmtKind::PackedClassDecl { .. }
            | StmtKind::ExternFunctionDecl { .. }
            | StmtKind::ExternClassDecl { .. }
            | StmtKind::ExternGlobalDecl { .. } => state,
        }
    }

    /// Executes `try`/`catch`/`finally`.
    fn exec_try(
        &mut self,
        try_body: &[Stmt],
        catches: &[crate::parser::ast::CatchClause],
        finally_body: Option<&[Stmt]>,
        state: State,
    ) -> State {
        let mut written = HashSet::new();
        collect_writes_stmts(try_body, &mut written);
        // An exception may leave the try body after any prefix of its writes.
        let mut catch_entry = state.clone();
        self.forget(&mut catch_entry, &written);
        let try_end = self.exec(try_body, state);
        let mut after = try_end;
        for catch in catches {
            let mut entry = catch_entry.clone();
            if let Some(variable) = &catch.variable {
                self.write(&mut entry, variable, AbsVal::Top);
            }
            let end = self.exec(&catch.body, entry);
            after = join_states(&after, &end);
        }
        match finally_body {
            Some(finally_body) => {
                // Variables written by a finally body are pinned for the whole function, so the
                // finally can run from any entry approximation without losing soundness.
                let mut entry = join_states(&after, &catch_entry);
                for catch in catches {
                    let mut catch_writes = HashSet::new();
                    collect_writes_stmts(&catch.body, &mut catch_writes);
                    self.forget(&mut entry, &catch_writes);
                }
                let end = self.exec(finally_body, entry);
                if after.is_some() { end } else { None }
            }
            None => after,
        }
    }

    /// Routes a `break`/`continue` state to its target frame.
    fn jump(&mut self, state: &State, levels: usize, is_continue: bool) {
        let levels = levels.max(1);
        if levels > self.frames.len() {
            return;
        }
        let index = self.frames.len() - levels;
        let frame = &mut self.frames[index];
        // `continue` targeting a `switch` behaves like `break`.
        if is_continue && !frame.is_switch {
            frame.continues = join_states(&frame.continues, state);
        } else {
            frame.breaks = join_states(&frame.breaks, state);
        }
    }

    /// Runs a loop-shaped construct to a fixpoint, then once more while recording.
    ///
    /// `body` receives the loop-head state with a fresh frame already pushed (it must pop it)
    /// and returns `(back_edge_state, exit_state)`.
    fn run_loop(
        &mut self,
        entry: State,
        is_switch: bool,
        body: &mut dyn FnMut(&mut Self, State) -> (State, State),
    ) -> State {
        let recording = std::mem::replace(&mut self.recording, false);
        let mut head = entry.clone();
        let mut iteration = 0;
        loop {
            self.frames.push(Frame {
                is_switch,
                ..Frame::default()
            });
            let (back, _) = body(self, head.clone());
            let joined = join_states(&head, &join_states(&entry, &back));
            let next = if iteration >= WIDEN_AFTER {
                widen_states(&head, &joined, &self.thresholds)
            } else {
                joined
            };
            if next == head {
                break;
            }
            head = next;
            iteration += 1;
            if iteration >= MAX_ITERATIONS {
                head = head.map(|_| Env::default());
                break;
            }
        }
        // Narrowing: re-applying the loop transfer to a post-fixpoint yields a smaller
        // post-fixpoint, which recovers bounds lost to widening (e.g. `$i` after `$i < 10`).
        for _ in 0..NARROWING_PASSES {
            self.frames.push(Frame {
                is_switch,
                ..Frame::default()
            });
            let (back, _) = body(self, head.clone());
            let next = join_states(&entry, &back);
            if next == head {
                break;
            }
            head = next;
        }
        self.recording = recording;
        self.frames.push(Frame {
            is_switch,
            ..Frame::default()
        });
        let (_, exit) = body(self, head);
        exit
    }

    // ----- conditions ----------------------------------------------------------------------

    /// Evaluates a condition, returning the states where it is truthy and falsy.
    fn cond(&mut self, expr: &Expr, mut state: State) -> (State, State) {
        match &expr.kind {
            ExprKind::BoolLiteral(true) => (state, None),
            ExprKind::BoolLiteral(false) => (None, state),
            ExprKind::Not(inner) => {
                let (truthy, falsy) = self.cond(inner, state);
                (falsy, truthy)
            }
            ExprKind::ErrorSuppress(inner) => self.cond(inner, state),
            ExprKind::BinaryOp { left, op: BinOp::And, right } => {
                let (left_true, left_false) = self.cond(left, state);
                let (right_true, right_false) = self.cond(right, left_true);
                (right_true, join_states(&left_false, &right_false))
            }
            ExprKind::BinaryOp { left, op: BinOp::Or, right } => {
                let (left_true, left_false) = self.cond(left, state);
                let (right_true, right_false) = self.cond(right, left_false);
                (join_states(&left_true, &right_true), right_false)
            }
            ExprKind::BinaryOp { left, op, right }
                if matches!(
                    op,
                    BinOp::Lt
                        | BinOp::LtEq
                        | BinOp::Gt
                        | BinOp::GtEq
                        | BinOp::Eq
                        | BinOp::StrictEq
                        | BinOp::NotEq
                        | BinOp::StrictNotEq
                ) && !has_writes(left)
                    && !has_writes(right) =>
            {
                let lv = self.eval(left, &mut state);
                let rv = self.eval(right, &mut state);
                self.narrow_comparison(op, left, lv, right, rv, state)
            }
            _ => {
                self.eval(expr, &mut state);
                (state.clone(), state)
            }
        }
    }

    /// Splits a state on an integer comparison between two definite ints.
    fn narrow_comparison(
        &self,
        op: &BinOp,
        left: &Expr,
        lv: AbsVal,
        right: &Expr,
        rv: AbsVal,
        state: State,
    ) -> (State, State) {
        let (Some(l), Some(r)) = (lv.bounds(), rv.bounds()) else {
            return (state.clone(), state);
        };
        // Normalize to `left < right`, `left <= right`, `left == right`, possibly negated.
        let (truthy, falsy) = match op {
            BinOp::Lt => (Cmp::Lt, Cmp::Ge),
            BinOp::LtEq => (Cmp::Le, Cmp::Gt),
            BinOp::Gt => (Cmp::Gt, Cmp::Le),
            BinOp::GtEq => (Cmp::Ge, Cmp::Lt),
            BinOp::Eq | BinOp::StrictEq => (Cmp::Eq, Cmp::Ne),
            _ => (Cmp::Ne, Cmp::Eq),
        };
        let left_name = self.narrowable(left);
        let right_name = self.narrowable(right);
        (
            apply_cmp(state.clone(), truthy, left_name, l, right_name, r),
            apply_cmp(state, falsy, left_name, l, right_name, r),
        )
    }

    /// Returns the variable an operand names when it can be narrowed.
    fn narrowable<'e>(&self, expr: &'e Expr) -> Option<&'e str> {
        match &expr.kind {
            ExprKind::Variable(name) if !self.is_pinned(name) => Some(name),
            _ => None,
        }
    }

    // ----- expressions ---------------------------------------------------------------------

    /// Evaluates operands left to right, first forgetting any variable that one operand writes
    /// and another reads (so the result holds for every evaluation order).
    fn eval_operands(&mut self, operands: &[&Expr], state: &mut State) -> Vec<AbsVal> {
        if operands.len() > 1 {
            let mut hazards = HashSet::new();
            for (index, operand) in operands.iter().enumerate() {
                let mut writes = HashSet::new();
                collect_writes_expr(operand, &mut writes);
                if writes.is_empty() {
                    continue;
                }
                for (other_index, other) in operands.iter().enumerate() {
                    if other_index == index {
                        continue;
                    }
                    let mut reads = HashSet::new();
                    collect_reads_expr(other, &mut reads);
                    collect_writes_expr(other, &mut reads);
                    hazards.extend(writes.intersection(&reads).cloned());
                }
            }
            self.forget(state, &hazards);
        }
        operands.iter().map(|operand| self.eval(operand, state)).collect()
    }

    /// Evaluates one expression, threading its side effects through `state`.
    fn eval(&mut self, expr: &Expr, state: &mut State) -> AbsVal {
        match &expr.kind {
            ExprKind::IntLiteral(value) => AbsVal::exact(*value),
            ExprKind::Variable(name) => self.read(state, name),
            ExprKind::BinaryOp { left, op, right } => self.eval_binary(expr, left, op, right, state),
            ExprKind::PreIncrement(name) => self.eval_step(expr, name, ArithKind::Inc, false, state),
            ExprKind::PostIncrement(name) => self.eval_step(expr, name, ArithKind::Inc, true, state),
            ExprKind::PreDecrement(name) => self.eval_step(expr, name, ArithKind::Dec, false, state),
            ExprKind::PostDecrement(name) => self.eval_step(expr, name, ArithKind::Dec, true, state),
            ExprKind::Negate(inner) => {
                let value = self.eval(inner, state);
                ops::negate(value.bounds())
            }
            ExprKind::BitNot(inner) => {
                let value = self.eval(inner, state);
                ops::bit_not(value.bounds())
            }
            ExprKind::Print(inner) => {
                self.eval(inner, state);
                AbsVal::exact(1)
            }
            ExprKind::ErrorSuppress(inner) => self.eval(inner, state),
            ExprKind::Cast { target, expr: inner } => {
                let value = self.eval(inner, state);
                match (target, value) {
                    (CastType::Int, AbsVal::Int(..)) => value,
                    (CastType::Int, AbsVal::Top) => AbsVal::FULL_INT,
                    _ => AbsVal::Top,
                }
            }
            ExprKind::Throw(inner) => {
                self.eval(inner, state);
                *state = None;
                AbsVal::Top
            }
            ExprKind::Assignment {
                target,
                value,
                result_target,
                prelude,
                conditional_value_temp,
            } => {
                let mut current = std::mem::take(state);
                current = self.exec(prelude, current);
                *state = current;
                let result = match &target.kind {
                    ExprKind::Variable(name) => {
                        let result = self.eval(value, state);
                        self.write(state, name, result);
                        result
                    }
                    _ => {
                        let values = self.eval_operands(&[value, target], state);
                        self.forget_root(state, target);
                        values[0]
                    }
                };
                if let Some(temp) = conditional_value_temp {
                    self.write(state, temp, AbsVal::Top);
                }
                match result_target {
                    Some(result_target) => self.eval(result_target, state),
                    None => result,
                }
            }
            ExprKind::ListUnpack { vars, value } => {
                self.eval(value, state);
                for var in vars {
                    self.write(state, var, AbsVal::Top);
                }
                AbsVal::Top
            }
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let (mut truthy, mut falsy) = self.cond(condition, std::mem::take(state));
                let then_value = self.eval(then_expr, &mut truthy);
                let else_value = self.eval(else_expr, &mut falsy);
                let value = join_opt(
                    truthy.is_some().then_some(then_value),
                    falsy.is_some().then_some(else_value),
                );
                *state = join_states(&truthy, &falsy);
                value.unwrap_or(AbsVal::Top)
            }
            ExprKind::ShortTernary { value, default } | ExprKind::NullCoalesce { value, default } => {
                self.eval_fallback(value, default, state)
            }
            ExprKind::Match {
                subject,
                arms,
                default,
            } => {
                self.eval(subject, state);
                let mut ends = None;
                let mut result = None;
                for (conditions, value) in arms {
                    for condition in conditions {
                        self.eval(condition, state);
                    }
                    let mut arm = state.clone();
                    let arm_value = self.eval(value, &mut arm);
                    if arm.is_some() {
                        result = join_opt(result, Some(arm_value));
                    }
                    ends = join_states(&ends, &arm);
                }
                if let Some(default) = default {
                    let mut arm = state.clone();
                    let arm_value = self.eval(default, &mut arm);
                    if arm.is_some() {
                        result = join_opt(result, Some(arm_value));
                    }
                    ends = join_states(&ends, &arm);
                }
                *state = ends;
                result.unwrap_or(AbsVal::Top)
            }
            ExprKind::FunctionCall { name, args } => self.eval_function_call(name.as_str(), args, state),
            ExprKind::Closure {
                params,
                body,
                capture_refs,
                ..
            } => {
                if self.recording {
                    self.analyze_function(None, params, body, capture_refs);
                }
                AbsVal::Top
            }
            ExprKind::Pipe { value, callable } => {
                self.eval_operands(&[value, callable], state);
                AbsVal::Top
            }
            ExprKind::NullsafeMethodCall { object, .. }
            | ExprKind::NullsafeDynamicMethodCall { object, .. }
            | ExprKind::NullsafeDynamicPropertyAccess { object, .. } => {
                // The rest of a nullsafe chain is skipped when the object is null.
                self.eval(object, state);
                let skipped = state.clone();
                let rest = nullsafe_rest(expr);
                self.eval_operands(&rest, state);
                *state = join_states(&skipped, state);
                AbsVal::Top
            }
            _ => {
                let mut children = Vec::new();
                super::scan::for_each_child(expr, &mut |child| {
                    if let super::scan::Child::Expr(child) = child {
                        children.push(child);
                    }
                });
                self.eval_operands(&children, state);
                AbsVal::Top
            }
        }
    }

    /// Evaluates `value ?: default` / `value ?? default`.
    fn eval_fallback(&mut self, value: &Expr, default: &Expr, state: &mut State) -> AbsVal {
        let first = self.eval(value, state);
        let mut fallback_state = state.clone();
        let second = self.eval(default, &mut fallback_state);
        *state = join_states(state, &fallback_state);
        first.join(second)
    }

    /// Evaluates a binary operator.
    fn eval_binary(&mut self, expr: &Expr, left: &Expr, op: &BinOp, right: &Expr, state: &mut State) -> AbsVal {
        match op {
            BinOp::And | BinOp::Or => {
                let (truthy, falsy) = self.cond(left, std::mem::take(state));
                let (mut evaluated, skipped) = if matches!(op, BinOp::And) {
                    (truthy, falsy)
                } else {
                    (falsy, truthy)
                };
                self.eval(right, &mut evaluated);
                *state = join_states(&evaluated, &skipped);
                AbsVal::Top
            }
            BinOp::NullCoalesce => self.eval_fallback(left, right, state),
            _ => {
                let values = self.eval_operands(&[left, right], state);
                let (l, r) = (values[0].bounds(), values[1].bounds());
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                        let kind = match op {
                            BinOp::Add => ArithKind::Add,
                            BinOp::Sub => ArithKind::Sub,
                            _ => ArithKind::Mul,
                        };
                        let result = match (l, r) {
                            (Some(l), Some(r)) => match op {
                                BinOp::Add => ops::add(l, r),
                                BinOp::Sub => ops::sub(l, r),
                                _ => ops::mul(l, r),
                            },
                            _ => None,
                        };
                        self.record(state, expr, kind, result.is_some());
                        result.unwrap_or(AbsVal::Top)
                    }
                    BinOp::Mod => ops::rem(l, r),
                    BinOp::BitAnd => ops::bit_and(l, r),
                    BinOp::BitOr | BinOp::BitXor => ops::bit_or_xor(l, r),
                    BinOp::ShiftLeft => ops::shl(l, r),
                    BinOp::ShiftRight => ops::shr(l, r),
                    BinOp::Spaceship => AbsVal::Int(-1, 1),
                    _ => AbsVal::Top,
                }
            }
        }
    }

    /// Evaluates `++`/`--` on a local.
    fn eval_step(&mut self, expr: &Expr, name: &str, kind: ArithKind, post: bool, state: &mut State) -> AbsVal {
        let old = self.read(state, name);
        let new = match old.bounds() {
            Some(bounds) => match kind {
                ArithKind::Inc => ops::add(bounds, (1, 1)),
                _ => ops::sub(bounds, (1, 1)),
            },
            None => None,
        };
        self.record(state, expr, kind, new.is_some());
        let new = new.unwrap_or(AbsVal::Top);
        self.write(state, name, new);
        if post { old } else { new }
    }

    /// Evaluates a direct function call and returns the range of its result.
    fn eval_function_call(&mut self, name: &str, args: &[Expr], state: &mut State) -> AbsVal {
        let key = symbol_key(name);
        if key == "unset" {
            for arg in args {
                match &arg.kind {
                    ExprKind::Variable(var) => self.write(state, var, AbsVal::Top),
                    _ => {
                        self.eval(arg, state);
                    }
                }
            }
            return AbsVal::Top;
        }
        let refs: Vec<&Expr> = args.iter().collect();
        let values = self.eval_operands(&refs, state);
        let positional = args.iter().all(|arg| !matches!(arg.kind, ExprKind::NamedArg { .. } | ExprKind::Spread(_)));
        if self.info.is_user_function(name) {
            if self.recording && state.is_some() && self.info.tracks_call_sites(name) {
                self.observe_call(&key, args, &values, positional);
            }
            return if self.info.user_function_returns_int(name) { AbsVal::FULL_INT } else { AbsVal::Top };
        }
        let ints = || values.iter().map(|value| value.bounds()).collect::<Option<Vec<_>>>();
        match key.as_str() {
            "count" | "sizeof" if positional && args.len() == 1 => AbsVal::Int(0, MEMORY_BOUND),
            "strlen" | "mb_strlen" if positional && args.len() == 1 => AbsVal::Int(0, MEMORY_BOUND),
            "ord" => AbsVal::Int(0, 255),
            "crc32" => AbsVal::Int(0, u32::MAX as i64),
            "mt_getrandmax" | "getrandmax" => AbsVal::exact(i32::MAX as i64),
            "mt_rand" | "rand" if args.is_empty() => AbsVal::Int(0, i32::MAX as i64),
            "mt_rand" | "rand" | "random_int" if positional && args.len() == 2 => match ints() {
                Some(bounds) => {
                    let lo = bounds[0].0.min(bounds[1].0);
                    let hi = bounds[0].1.max(bounds[1].1);
                    AbsVal::Int(lo, hi)
                }
                None => AbsVal::FULL_INT,
            },
            "min" | "max" if positional && args.len() >= 2 => match ints() {
                Some(bounds) => {
                    let fold = |pick: fn(i64, i64) -> i64, side: fn(&(i64, i64)) -> i64| {
                        bounds.iter().map(side).reduce(pick).expect("two or more arguments")
                    };
                    if key == "min" {
                        AbsVal::Int(fold(i64::min, |b| b.0), fold(i64::min, |b| b.1))
                    } else {
                        AbsVal::Int(fold(i64::max, |b| b.0), fold(i64::max, |b| b.1))
                    }
                }
                None => AbsVal::Top,
            },
            "abs" if positional && args.len() == 1 => match values[0].bounds() {
                Some((lo, hi)) if lo > MIN => {
                    let (alo, ahi) = (lo.abs(), hi.abs());
                    let low = if lo <= 0 && hi >= 0 { 0 } else { alo.min(ahi) };
                    AbsVal::Int(low, alo.max(ahi))
                }
                _ => AbsVal::Top,
            },
            _ => match crate::types::builtin_call_sig(&key) {
                Some(sig) if sig.return_type == crate::types::PhpType::Int => AbsVal::FULL_INT,
                _ => AbsVal::Top,
            },
        }
    }

    /// Joins one reachable direct call's argument values into the callee's observations.
    ///
    /// Named or spread arguments make every position unknown. Omitted trailing arguments are
    /// recorded as unknown too (their defaults are not modelled).
    fn observe_call(&mut self, key: &str, args: &[Expr], values: &[AbsVal], positional: bool) {
        let arity = self.info.function_arity(key);
        let entry = self
            .calls
            .observed
            .entry(key.to_string())
            .or_insert_with(|| vec![None; arity]);
        for (index, slot) in entry.iter_mut().enumerate() {
            let value = if positional && index < args.len() { values[index] } else { AbsVal::Top };
            *slot = Some(slot.map_or(value, |seen| seen.join(value)));
        }
    }

    /// Forgets the variable at the root of an lvalue.
    fn forget_root(&self, state: &mut State, target: &Expr) {
        if let Some(name) = root_variable(target) {
            self.write(state, name, AbsVal::Top);
        }
    }
}

/// Collects the widening thresholds of a body: its integer literals, their negations, and the
/// neighbours of both (loop bounds are typically `n`, `n - 1` or `n + 1` away from a literal).
fn widening_thresholds(body: &[Stmt]) -> Vec<i64> {
    let mut values = std::collections::BTreeSet::from([-1, 0, 1]);
    walk_stmts(body, false, &mut |node| {
        if let Node::Expr(Expr {
            kind: ExprKind::IntLiteral(value),
            ..
        }) = node
        {
            for base in [*value, value.saturating_neg()] {
                values.insert(base);
                values.insert(base.saturating_sub(1));
                values.insert(base.saturating_add(1));
            }
        }
    });
    values.into_iter().collect()
}

/// Operands of a nullsafe node that only run when the object is not null.
fn nullsafe_rest(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::NullsafeMethodCall { args, .. } => args.iter().collect(),
        ExprKind::NullsafeDynamicMethodCall { method, args, .. } => {
            std::iter::once(&**method).chain(args.iter()).collect()
        }
        ExprKind::NullsafeDynamicPropertyAccess { property, .. } => vec![&**property],
        _ => Vec::new(),
    }
}

/// Returns `true` when evaluating `expr` may write a local.
fn has_writes(expr: &Expr) -> bool {
    let mut writes = HashSet::new();
    collect_writes_expr(expr, &mut writes);
    !writes.is_empty()
}

/// Comparison relation between the left and right operand.
#[derive(Clone, Copy)]
enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

/// Narrows both operands of `left <cmp> right` (each only when it is a narrowable variable).
fn apply_cmp(
    state: State,
    cmp: Cmp,
    left_name: Option<&str>,
    l: (i64, i64),
    right_name: Option<&str>,
    r: (i64, i64),
) -> State {
    // `a < b` holds for some values in the intervals only when a.lo < b.hi, and so on; an
    // impossible relation makes the branch unreachable.
    let feasible = match cmp {
        Cmp::Lt => l.0 < r.1,
        Cmp::Le => l.0 <= r.1,
        Cmp::Gt => l.1 > r.0,
        Cmp::Ge => l.1 >= r.0,
        Cmp::Eq => l.0 <= r.1 && r.0 <= l.1,
        Cmp::Ne => !(l.0 == l.1 && r.0 == r.1 && l.0 == r.0),
    };
    if !feasible {
        return None;
    }
    let mut state = state;
    let (left_bounds, right_bounds) = match cmp {
        Cmp::Lt => ((MIN, r.1.saturating_sub(1)), (l.0.saturating_add(1), MAX)),
        Cmp::Le => ((MIN, r.1), (l.0, MAX)),
        Cmp::Gt => ((r.0.saturating_add(1), MAX), (MIN, l.1.saturating_sub(1))),
        Cmp::Ge => ((r.0, MAX), (MIN, l.1)),
        Cmp::Eq => ((r.0, r.1), (l.0, l.1)),
        Cmp::Ne => {
            let exclude = |own: (i64, i64), other: (i64, i64)| {
                if other.0 != other.1 {
                    return own;
                }
                let value = other.0;
                if own.0 == value {
                    (value.saturating_add(1), own.1)
                } else if own.1 == value {
                    (own.0, value.saturating_sub(1))
                } else {
                    own
                }
            };
            (exclude(l, r), exclude(r, l))
        }
    };
    if let Some(name) = left_name {
        state = narrow(state, name, left_bounds.0, left_bounds.1);
    }
    if let Some(name) = right_name {
        state = narrow(state, name, right_bounds.0, right_bounds.1);
    }
    state
}
