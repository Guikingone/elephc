//! Purpose:
//! Exercises range-domain soundness and static expression graph complexity.
//!
//! Called from:
//! - The integer-range pass unit-test harness.
//!
//! Key details:
//! - Unsupported shared expressions must be cached as unknown.
//! - Deep graphs must not consume the native call stack.

use super::*;
use crate::ir::Builder;

/// Builds a shared unsupported expression graph and requires unknown results to be memoized.
#[test]
fn shared_unknown_expression_is_memoized() {
    let mut function = Function::new("shared".to_string(), IrType::F64, PhpType::Float);
    let result;
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let mut value = builder.emit_const_f64(1.0);
        for _ in 0..16 {
            value = builder.emit(
                Op::FAdd, vec![value, value], None,
                IrType::F64, PhpType::Float, Ownership::NonHeap,
            ).expect("float addition");
        }
        result = value;
        builder.terminate(Terminator::Return { value: Some(value) });
    }
    let mut memo = HashMap::new();
    assert_eq!(static_value_range(
        &function, result, &HashMap::new(), &mut memo,
    ), None);
    assert_eq!(memo.len(), 17, "unknown subexpressions must be memoized too");
}

/// A long linear dependency chain uses the heap worklist instead of recursive stack frames.
#[test]
fn deep_expression_uses_bounded_call_stack() {
    let mut function = Function::new("deep".to_string(), IrType::I64, PhpType::Int);
    let result;
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let one = builder.emit_const_i64(1);
        let mut value = one;
        for _ in 0..20_000 {
            value = builder.emit(
                Op::IAdd, vec![value, one], None,
                IrType::I64, PhpType::Int, Ownership::NonHeap,
            ).expect("integer addition");
        }
        result = value;
        builder.terminate(Terminator::Return { value: Some(value) });
    }
    let mut memo = HashMap::new();
    assert_eq!(static_value_range(
        &function, result, &HashMap::new(), &mut memo,
    ), Some(IntRange::point(20_001)));
    assert_eq!(memo.len(), 20_001);
}

/// Samples narrow and wide intervals around zero, powers of two, and signed overflow limits.
fn sample_ranges() -> Vec<IntRange> {
    let points = [i64::MIN, i64::MIN + 1, -256, -16, -1, 0, 1, 16, 255, i64::MAX - 1, i64::MAX];
    let mut ranges: Vec<_> = points.iter().copied().map(IntRange::point).collect();
    ranges.extend(points.windows(2).map(|ends| IntRange { lo: ends[0], hi: ends[1] }));
    ranges.push(IntRange::full());
    ranges
}

/// Chooses endpoints and an overflow-safe midpoint from one inclusive interval.
fn samples(range: IntRange) -> [i64; 3] {
    [range.lo, ((range.lo as i128 + range.hi as i128) / 2) as i64, range.hi]
}

/// Checks that a mathematical result lies in the interval asserted by the analysis.
fn assert_contains(range: IntRange, value: i128) {
    assert!(range.lo as i128 <= value && value <= range.hi as i128, "{value} outside {range:?}");
}

/// Arithmetic and bitwise transfer facts contain every sampled concrete result.
#[test]
fn sampled_transfer_ranges_are_sound() {
    let ranges = sample_ranges();
    for &lhs in &ranges {
        for &rhs in &ranges {
            for x in samples(lhs) {
                for y in samples(rhs) {
                    for (range, actual) in [
                        (checked_add(lhs, rhs), x as i128 + y as i128),
                        (checked_sub(lhs, rhs), x as i128 - y as i128),
                        (checked_mul(lhs, rhs), x as i128 * y as i128),
                        (bitand_range(lhs, rhs), (x & y) as i128),
                        (bitor_range(lhs, rhs), (x | y) as i128),
                        (bitxor_range(lhs, rhs), (x ^ y) as i128),
                    ] {
                        if let Some(range) = range {
                            assert_contains(range, actual);
                        }
                    }
                }
            }
        }
        for count in 0..=63 {
            for value in samples(lhs) {
                if let Some(range) = shift_left_range(lhs, IntRange::point(count)) {
                    assert_contains(range, value.wrapping_shl(count as u32) as i128);
                }
                if let Some(range) = shift_right_range(lhs, IntRange::point(count)) {
                    assert_contains(range, (value >> count) as i128);
                }
            }
        }
    }
}

/// Proven induction summaries contain all sampled headers and never permit a wrapping update.
#[test]
fn sampled_induction_summaries_are_sound() {
    let ranges = sample_ranges();
    for &init in &ranges {
        for &bound in &ranges {
            for step in [-16, -3, -1, 1, 3, 16] {
                for predicate in [CmpPredicate::Slt, CmpPredicate::Sle, CmpPredicate::Sgt, CmpPredicate::Sge] {
                    let Some(summary) = induction_summary(init, step, predicate, bound) else {
                        continue;
                    };
                    for initial in samples(init) {
                        for limit in samples(bound) {
                            let mut value = initial;
                            for _ in 0..32 {
                                assert_contains(summary, value as i128);
                                let continues = match predicate {
                                    CmpPredicate::Slt => value < limit,
                                    CmpPredicate::Sle => value <= limit,
                                    CmpPredicate::Sgt => value > limit,
                                    CmpPredicate::Sge => value >= limit,
                                    _ => unreachable!(),
                                };
                                if !continues {
                                    break;
                                }
                                value = value.checked_add(step).expect("proven recurrence overflowed");
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Signed comparison refinements never discard a satisfying pair at domain boundaries.
#[test]
fn sampled_comparison_refinements_are_sound() {
    let ranges = sample_ranges();
    for &lhs in &ranges {
        for &rhs in &ranges {
            for predicate in [
                CmpPredicate::Eq, CmpPredicate::Ne, CmpPredicate::Slt,
                CmpPredicate::Sle, CmpPredicate::Sgt, CmpPredicate::Sge,
            ] {
                for x in samples(lhs) {
                    for y in samples(rhs) {
                        let satisfies = match predicate {
                            CmpPredicate::Eq => x == y,
                            CmpPredicate::Ne => x != y,
                            CmpPredicate::Slt => x < y,
                            CmpPredicate::Sle => x <= y,
                            CmpPredicate::Sgt => x > y,
                            CmpPredicate::Sge => x >= y,
                            _ => unreachable!(),
                        };
                        if satisfies {
                            let (left, right) = refine_ranges(lhs, predicate, rhs)
                                .expect("satisfying comparison marked unreachable");
                            assert_contains(left, x as i128);
                            assert_contains(right, y as i128);
                        }
                    }
                }
            }
        }
    }
}
