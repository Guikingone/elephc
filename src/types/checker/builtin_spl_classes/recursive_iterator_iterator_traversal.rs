//! Purpose:
//! Builds RecursiveIteratorIterator traversal-state synthetic method bodies.
//! Keeps recursive frame-stack transitions away from class declaration wiring.
//!
//! Called from:
//! - `super::recursive_iterator_iterator` method declarations.
//!
//! Key details:
//! - State arrays track iterators, frame states, and frame depths in parallel.
//! - Advance logic preserves self-first, child-first, and leaves-only traversal modes.

use crate::parser::ast::{BinOp, Expr, Stmt, TypeExpr};

use super::common::*;
use super::recursive_array::assume_recursive_iterator_expr;

/// Builds the AST expression for recursive iterator iterator root.
fn recursive_iterator_iterator_root_expr() -> Expr {
    property_access(this_expr(), "root")
}

/// Builds the AST expression for recursive iterator iterator mode.
fn recursive_iterator_iterator_mode_expr() -> Expr {
    property_access(this_expr(), "mode")
}

/// Builds the AST expression for recursive iterator iterator iterators.
fn recursive_iterator_iterator_iterators_expr() -> Expr {
    property_access(this_expr(), "iterators")
}

/// Builds the AST expression for recursive iterator iterator states.
fn recursive_iterator_iterator_states_expr() -> Expr {
    property_access(this_expr(), "states")
}

/// Builds the AST expression for recursive iterator iterator depths.
fn recursive_iterator_iterator_depths_expr() -> Expr {
    property_access(this_expr(), "depths")
}

/// Builds the AST expression for recursive iterator iterator depth.
fn recursive_iterator_iterator_depth_expr() -> Expr {
    property_access(this_expr(), "depth")
}

/// Builds the AST expression for recursive iterator iterator slot.
fn recursive_iterator_iterator_slot_expr() -> Expr {
    property_access(this_expr(), "slot")
}

/// Builds the AST expression for recursive iterator iterator max depth (`-1` = unlimited).
fn recursive_iterator_iterator_max_depth_expr() -> Expr {
    property_access(this_expr(), "maxDepth")
}

/// Builds the AST expression for recursive iterator iterator current valid.
fn recursive_iterator_iterator_current_valid_expr() -> Expr {
    property_access(this_expr(), "currentValid")
}

/// Builds the AST expression for recursive iterator iterator valid.
fn recursive_iterator_iterator_valid_expr() -> Expr {
    recursive_iterator_iterator_current_valid_expr()
}

/// Provides the Recursive iterator iterator iterator at depth helper used by the recursive iterator iterator traversal module.
fn recursive_iterator_iterator_iterator_at_depth(depth: Expr) -> Expr {
    array_access(
        recursive_iterator_iterator_iterators_expr(),
        depth,
    )
}

/// Provides the Recursive iterator iterator state at current slot helper used by the recursive iterator iterator traversal module.
fn recursive_iterator_iterator_state_at_current_slot() -> Expr {
    array_access(
        recursive_iterator_iterator_states_expr(),
        recursive_iterator_iterator_slot_expr(),
    )
}

/// Provides the Recursive iterator iterator depth at current slot helper used by the recursive iterator iterator traversal module.
fn recursive_iterator_iterator_depth_at_current_slot() -> Expr {
    array_access(
        recursive_iterator_iterator_depths_expr(),
        recursive_iterator_iterator_slot_expr(),
    )
}

/// Builds the AST expression for recursive iterator iterator current iterator.
fn recursive_iterator_iterator_current_iterator_expr() -> Expr {
    recursive_iterator_iterator_iterator_at_depth(recursive_iterator_iterator_slot_expr())
}

/// Builds the AST expression for recursive iterator iterator slot for depth.
fn recursive_iterator_iterator_slot_for_depth_expr(depth: Expr) -> Expr {
    method_call(this_expr(), "__elephcSlotForDepth", vec![depth])
}

/// Builds the synthetic method body for recursive iterator iterator construct.
pub(super) fn recursive_iterator_iterator_construct_body() -> Vec<Stmt> {
    vec![
        property_assign_stmt(this_expr(), "root", var_expr("iterator")),
        property_assign_stmt(this_expr(), "mode", var_expr("mode")),
        property_assign_stmt(this_expr(), "flags", var_expr("flags")),
        property_assign_stmt(this_expr(), "iterators", empty_array_expr()),
        property_assign_stmt(this_expr(), "states", empty_array_expr()),
        property_assign_stmt(this_expr(), "depths", empty_array_expr()),
        property_assign_stmt(this_expr(), "depth", int_expr(0)),
        property_assign_stmt(this_expr(), "slot", int_expr(0)),
        property_assign_stmt(this_expr(), "currentValid", bool_expr(false)),
        // PHP starts every RecursiveIteratorIterator unlimited; `setMaxDepth()` narrows it.
        property_assign_stmt(this_expr(), "maxDepth", int_expr(-1)),
    ]
}

/// Builds the synthetic method body for `RecursiveIteratorIterator::setMaxDepth()`.
///
/// PHP validates the argument before storing it: `setMaxDepth(-2)` throws
/// `ValueError: RecursiveIteratorIterator::setMaxDepth(): Argument #1 ($maxDepth) must be greater
/// than or equal to -1` (verified against php 8.5.6). `-1` (the default) means unlimited.
pub(super) fn recursive_iterator_iterator_set_max_depth_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            binary_expr(var_expr("maxDepth"), BinOp::Lt, int_expr(-1)),
            vec![throw_stmt(new_object_expr(
                "ValueError",
                vec![string_expr(
                    "RecursiveIteratorIterator::setMaxDepth(): Argument #1 ($maxDepth) must be greater than or equal to -1",
                )],
            ))],
            None,
        ),
        property_assign_stmt(this_expr(), "maxDepth", var_expr("maxDepth")),
    ]
}

/// Builds the synthetic method body for `RecursiveIteratorIterator::getMaxDepth()`.
///
/// PHP returns `int|false`: `false` when any depth is allowed, otherwise the configured maximum
/// (verified against php 8.5.6 — a fresh iterator reports `bool(false)`, and `setMaxDepth(0)` then
/// reports `int(0)`). The internal `-1` sentinel is mapped back to `false` here.
pub(super) fn recursive_iterator_iterator_get_max_depth_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            binary_expr(
                recursive_iterator_iterator_max_depth_expr(),
                BinOp::Lt,
                int_expr(0),
            ),
            return_body(bool_expr(false)),
            None,
        ),
        return_stmt(recursive_iterator_iterator_max_depth_expr()),
    ]
}

/// Builds the synthetic method body for the internal descend gate.
///
/// True when the traversal may still descend from the CURRENT frame into its children: either no
/// maximum is configured (`maxDepth < 0`), or the current depth is strictly below it. PHP yields
/// the node sitting at `maxDepth` (it is simply treated as having no children) but never opens it,
/// so entries exist at depths `0..=maxDepth` — verified against php 8.5.6 on a 3-level tree:
/// `setMaxDepth(0)` yields only depth-0 entries, `setMaxDepth(1)` yields depths 0 and 1.
pub(super) fn recursive_iterator_iterator_may_descend_body() -> Vec<Stmt> {
    return_body(binary_expr(
        binary_expr(
            recursive_iterator_iterator_max_depth_expr(),
            BinOp::Lt,
            int_expr(0),
        ),
        BinOp::Or,
        binary_expr(
            recursive_iterator_iterator_depth_expr(),
            BinOp::Lt,
            recursive_iterator_iterator_max_depth_expr(),
        ),
    ))
}

/// Builds the synthetic method body for recursive iterator iterator rewind.
pub(super) fn recursive_iterator_iterator_rewind_body() -> Vec<Stmt> {
    vec![
        property_assign_stmt(this_expr(), "iterators", empty_array_expr()),
        property_assign_stmt(this_expr(), "states", empty_array_expr()),
        property_assign_stmt(this_expr(), "depths", empty_array_expr()),
        property_assign_stmt(this_expr(), "depth", int_expr(0)),
        property_assign_stmt(this_expr(), "slot", int_expr(0)),
        property_assign_stmt(this_expr(), "currentValid", bool_expr(false)),
        expr_stmt(method_call(
            recursive_iterator_iterator_root_expr(),
            "rewind",
            Vec::new(),
        )),
        if_stmt(
            method_call(recursive_iterator_iterator_root_expr(), "valid", Vec::new()),
            vec![
                property_array_push_stmt(this_expr(), "iterators", recursive_iterator_iterator_root_expr()),
                property_array_push_stmt(this_expr(), "states", int_expr(0)),
                property_array_push_stmt(this_expr(), "depths", int_expr(0)),
                expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
            ],
            None,
        ),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator valid.
pub(super) fn recursive_iterator_iterator_valid_body() -> Vec<Stmt> {
    return_body(recursive_iterator_iterator_valid_expr())
}

/// Builds the synthetic method body for recursive iterator iterator current.
pub(super) fn recursive_iterator_iterator_current_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            not_expr(recursive_iterator_iterator_valid_expr()),
            null_return_body(),
            None,
        ),
        return_stmt(method_call(
            recursive_iterator_iterator_current_iterator_expr(),
            "current",
            Vec::new(),
        )),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator key.
pub(super) fn recursive_iterator_iterator_key_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            not_expr(recursive_iterator_iterator_valid_expr()),
            null_return_body(),
            None,
        ),
        return_stmt(method_call(
            recursive_iterator_iterator_current_iterator_expr(),
            "key",
            Vec::new(),
        )),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator next.
pub(super) fn recursive_iterator_iterator_next_body() -> Vec<Stmt> {
    vec![if_stmt(
        recursive_iterator_iterator_valid_expr(),
        vec![expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new()))],
        None,
    )]
}

/// Builds the synthetic method body for recursive iterator iterator get depth.
pub(super) fn recursive_iterator_iterator_get_depth_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            not_expr(recursive_iterator_iterator_valid_expr()),
            return_body(int_expr(0)),
            None,
        ),
        return_stmt(recursive_iterator_iterator_depth_expr()),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator get inner iterator.
pub(super) fn recursive_iterator_iterator_get_inner_iterator_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            recursive_iterator_iterator_valid_expr(),
            return_body(recursive_iterator_iterator_current_iterator_expr()),
            None,
        ),
        return_stmt(recursive_iterator_iterator_root_expr()),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator get sub iterator.
pub(super) fn recursive_iterator_iterator_get_sub_iterator_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            not_expr(recursive_iterator_iterator_valid_expr()),
            vec![
                if_stmt(
                    binary_expr(var_expr("level"), BinOp::LtEq, int_expr(0)),
                    return_body(recursive_iterator_iterator_root_expr()),
                    None,
                ),
                return_stmt(null_expr()),
            ],
            None,
        ),
        if_stmt(
            binary_expr(var_expr("level"), BinOp::Lt, int_expr(0)),
            return_body(recursive_iterator_iterator_current_iterator_expr()),
            None,
        ),
        if_stmt(
            binary_expr(
                var_expr("level"),
                BinOp::LtEq,
                recursive_iterator_iterator_depth_expr(),
            ),
            return_body(recursive_iterator_iterator_iterator_at_depth(
                recursive_iterator_iterator_slot_for_depth_expr(var_expr("level")),
            )),
            None,
        ),
        return_stmt(null_expr()),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator slot for depth.
pub(super) fn recursive_iterator_iterator_slot_for_depth_body() -> Vec<Stmt> {
    vec![
        typed_assign_stmt("i", TypeExpr::Int, int_expr(0)),
        typed_assign_stmt("slot", TypeExpr::Int, int_expr(0)),
        typed_assign_stmt("limit", TypeExpr::Int, count_expr(recursive_iterator_iterator_depths_expr())),
        while_stmt(
            binary_expr(var_expr("i"), BinOp::Lt, var_expr("limit")),
            vec![
                if_stmt(
                    binary_expr(
                        array_access(recursive_iterator_iterator_depths_expr(), var_expr("i")),
                        BinOp::StrictEq,
                        var_expr("level"),
                    ),
                    vec![assign_stmt("slot", var_expr("i"))],
                    None,
                ),
                increment_stmt("i"),
            ],
        ),
        return_stmt(var_expr("slot")),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator advance.
pub(super) fn recursive_iterator_iterator_advance_body() -> Vec<Stmt> {
    vec![
        property_assign_stmt(this_expr(), "currentValid", bool_expr(false)),
        property_assign_stmt(
            this_expr(),
            "depth",
            recursive_iterator_iterator_depth_at_current_slot(),
        ),
        assign_stmt("iterator", recursive_iterator_iterator_current_iterator_expr()),
        if_stmt(
            not_expr(method_call(var_expr("iterator"), "valid", Vec::new())),
            recursive_iterator_iterator_pop_invalid_frame_body(),
            None,
        ),
        assign_stmt(
            "state",
            recursive_iterator_iterator_state_at_current_slot(),
        ),
        if_stmt(
            binary_expr(var_expr("state"), BinOp::StrictEq, int_expr(2)),
            vec![
                expr_stmt(method_call(var_expr("iterator"), "next", Vec::new())),
                property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(0)),
                expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
                return_void_stmt(),
            ],
            None,
        ),
        if_stmt(
            binary_expr(
                recursive_iterator_iterator_mode_expr(),
                BinOp::StrictEq,
                int_expr(1),
            ),
            recursive_iterator_iterator_advance_self_first_body(),
            Some(recursive_iterator_iterator_advance_children_first_or_leaves_body()),
        ),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator pop invalid frame.
fn recursive_iterator_iterator_pop_invalid_frame_body() -> Vec<Stmt> {
    vec![
        if_stmt(
            binary_expr(recursive_iterator_iterator_depth_expr(), BinOp::StrictEq, int_expr(0)),
            vec![
                property_assign_stmt(this_expr(), "currentValid", bool_expr(false)),
                return_void_stmt(),
            ],
            Some(vec![
                typed_assign_stmt(
                    "previousDepth",
                    TypeExpr::Int,
                    binary_expr(recursive_iterator_iterator_depth_expr(), BinOp::Sub, int_expr(1)),
                ),
                property_assign_stmt(
                    this_expr(),
                    "depth",
                    var_expr("previousDepth"),
                ),
                property_assign_stmt(
                    this_expr(),
                    "slot",
                    recursive_iterator_iterator_slot_for_depth_expr(var_expr("previousDepth")),
                ),
                expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
                return_void_stmt(),
            ]),
        ),
    ]
}

/// Builds the synthetic method body for recursive iterator iterator advance self first.
fn recursive_iterator_iterator_advance_self_first_body() -> Vec<Stmt> {
    let mut body = vec![if_stmt(
        binary_expr(var_expr("state"), BinOp::StrictEq, int_expr(0)),
        vec![
            property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(1)),
            property_assign_stmt(this_expr(), "currentValid", bool_expr(true)),
            return_void_stmt(),
        ],
        None,
    )];
    body.extend(vec![
        assign_stmt(
            "hasChildren",
            method_call(var_expr("iterator"), "hasChildren", Vec::new()),
        ),
        if_stmt(
            var_expr("hasChildren"),
            // SELF_FIRST already yielded this node on entry (state 0 above), so a blocked descend
            // simply falls through to the next sibling — no extra depth-limit case is needed here.
            vec![if_stmt(
                recursive_iterator_iterator_may_descend_expr(),
                vec![
                    assign_stmt("child", method_call(var_expr("iterator"), "getChildren", Vec::new())),
                    if_stmt(
                        not_expr(function_call("is_null", vec![var_expr("child")])),
                        recursive_iterator_iterator_descend_current_child_body(int_expr(2)),
                        None,
                    ),
                ],
                None,
            )],
            None,
        ),
        property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(2)),
        expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
        return_void_stmt(),
    ]);
    body
}

/// Builds `$this->__elephcMayDescend()` — the `maxDepth` descend gate.
///
/// It gates the DESCENT only, never `hasChildren()` itself: PHP keeps treating a depth-limited
/// node as "has children" for the LEAVES_ONLY emit decision, so collapsing the two would wrongly
/// emit it. Verified against php 8.5.6 on `[1, [2, [3, 4]], 5]` with `setMaxDepth(0)`:
/// LEAVES_ONLY yields `1 5` (the nested array is skipped, not emitted as a leaf), while SELF_FIRST
/// and CHILD_FIRST both yield the nested array itself.
fn recursive_iterator_iterator_may_descend_expr() -> Expr {
    method_call(this_expr(), "__elephcMayDescend", Vec::new())
}

/// Builds the synthetic method body for recursive iterator iterator advance children first or leaves.
fn recursive_iterator_iterator_advance_children_first_or_leaves_body() -> Vec<Stmt> {
    let mut body = vec![if_stmt(
        binary_expr(var_expr("state"), BinOp::StrictEq, int_expr(1)),
        vec![
            property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(2)),
            property_assign_stmt(this_expr(), "currentValid", bool_expr(true)),
            return_void_stmt(),
        ],
        None,
    )];
    body.extend(vec![
        assign_stmt(
            "hasChildren",
            method_call(var_expr("iterator"), "hasChildren", Vec::new()),
        ),
        if_stmt(
            var_expr("hasChildren"),
            vec![if_stmt(
                recursive_iterator_iterator_may_descend_expr(),
                vec![
                    assign_stmt("child", method_call(var_expr("iterator"), "getChildren", Vec::new())),
                    if_stmt(
                        not_expr(function_call("is_null", vec![var_expr("child")])),
                        recursive_iterator_iterator_non_self_child_body(),
                        None,
                    ),
                ],
                // Depth limit reached with children present. LEAVES_ONLY must NOT emit this node
                // (it is not a leaf — it still has children), so skip straight to the next
                // sibling; the identical skip lives in `..._non_self_child_body` for the
                // descended case. CHILD_FIRST falls through and emits the node, matching PHP.
                Some(vec![if_stmt(
                    binary_expr(
                        recursive_iterator_iterator_mode_expr(),
                        BinOp::StrictEq,
                        int_expr(0),
                    ),
                    vec![
                        property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(2)),
                        expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
                        return_void_stmt(),
                    ],
                    None,
                )]),
            )],
            None,
        ),
        property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(2)),
        property_assign_stmt(this_expr(), "currentValid", bool_expr(true)),
        return_void_stmt(),
    ]);
    body
}

/// Builds the synthetic method body for recursive iterator iterator non self child.
fn recursive_iterator_iterator_non_self_child_body() -> Vec<Stmt> {
    let mut body = recursive_iterator_iterator_descend_current_child_body(int_expr(2));
    body.push(if_stmt(
        binary_expr(
            recursive_iterator_iterator_mode_expr(),
            BinOp::StrictEq,
            int_expr(0),
        ),
        vec![
            property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(2)),
            expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
            return_void_stmt(),
        ],
        None,
    ));
    body
}

/// Builds the synthetic method body for recursive iterator iterator descend current child.
fn recursive_iterator_iterator_descend_current_child_body(parent_state: Expr) -> Vec<Stmt> {
    vec![
        assign_stmt(
            "recursiveChild",
            assume_recursive_iterator_expr(var_expr("child")),
        ),
        expr_stmt(method_call(var_expr("recursiveChild"), "rewind", Vec::new())),
        if_stmt(
            method_call(var_expr("recursiveChild"), "valid", Vec::new()),
            vec![
                if_stmt(
                    binary_expr(
                        recursive_iterator_iterator_mode_expr(),
                        BinOp::StrictEq,
                        int_expr(2),
                    ),
                    vec![property_array_assign_stmt(this_expr(), "states", recursive_iterator_iterator_slot_expr(), int_expr(1))],
                    Some(vec![property_array_assign_stmt(
                        this_expr(),
                        "states",
                        recursive_iterator_iterator_slot_expr(),
                        parent_state,
                    )]),
                ),
                typed_assign_stmt(
                    "nextDepth",
                    TypeExpr::Int,
                    binary_expr(recursive_iterator_iterator_depth_expr(), BinOp::Add, int_expr(1)),
                ),
                typed_assign_stmt("nextSlot", TypeExpr::Int, count_expr(recursive_iterator_iterator_iterators_expr())),
                property_array_push_stmt(this_expr(), "iterators", var_expr("recursiveChild")),
                property_array_push_stmt(this_expr(), "states", int_expr(0)),
                property_array_push_stmt(this_expr(), "depths", var_expr("nextDepth")),
                property_assign_stmt(this_expr(), "depth", var_expr("nextDepth")),
                property_assign_stmt(this_expr(), "slot", var_expr("nextSlot")),
                expr_stmt(method_call(this_expr(), "__elephcAdvance", Vec::new())),
                return_void_stmt(),
            ],
            None,
        ),
    ]
}
