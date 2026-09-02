//! Purpose:
//! Parser tests for comparison, equality, logical, ternary, match, and unary expressions.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - These cases focus on PHP precedence and associativity in EvalIR.

use super::support::*;

/// Verifies attributed arrow functions retain metadata and infer by-value captures.
#[test]
fn parse_fragment_accepts_attributed_arrow_closure() {
    let program = parse_fragment(
        br#"return #[\Closure(name: "service", class: "Demo\\Service")] fn ($value) => $container->load($value);"#,
    )
    .expect("attributed arrow function should parse");
    let [EvalStmt::Return(Some(EvalExpr::Closure {
        function,
        captures,
        is_static,
    }))] = program.statements()
    else {
        panic!("expected attributed arrow closure return");
    };
    assert!(!is_static);
    assert_eq!(function.attributes().len(), 1);
    assert_eq!(function.attributes()[0].name(), "Closure");
    assert_eq!(captures, &[EvalClosureCapture::new("container", false)]);
}

/// Verifies straight-line keyed yields lower to an iterator-returning eval closure.
#[test]
fn parse_fragment_lowers_straight_line_yields_to_array_iterator() {
    let program = parse_fragment(
        br#"return function () { yield 2 => "two"; yield 5 => "five"; };"#,
    )
    .expect("straight-line yields should parse");
    let [EvalStmt::Return(Some(EvalExpr::Closure { function, .. }))] = program.statements() else {
        panic!("expected generator closure return");
    };
    assert!(matches!(
        function.body(),
        [EvalStmt::Return(Some(EvalExpr::NewObject { class_name, args }))]
            if class_name == "ArrayIterator" && args.len() == 1
    ));
}

/// Verifies PHP's error-suppression prefix preserves the wrapped call expression.
#[test]
fn parse_fragment_accepts_error_suppression_source() {
    let program = parse_fragment(br#"return @trigger_error("deprecated", 16384);"#)
        .expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Unary {
            op: EvalUnaryOp::ErrorSuppress,
            expr: Box::new(EvalExpr::Call {
                name: "trigger_error".to_string(),
                args: vec![
                    EvalCallArg::positional(EvalExpr::Const(EvalConst::String(
                        "deprecated".to_string(),
                    ))),
                    EvalCallArg::positional(EvalExpr::Const(EvalConst::Int(16384))),
                ],
            }),
        }))]
    );
}

/// Verifies a conditionally declared typed variadic function accepts a suppressed call body.
#[test]
fn parse_fragment_accepts_conditional_typed_variadic_deprecation_function() {
    parse_fragment(
        br#"if (!function_exists('trigger_deprecation')) {
function trigger_deprecation(string $package, string $version, string $message, mixed ...$args): void {
    @trigger_error(($package || $version ? "Since $package $version: " : '').($args ? vsprintf($message, $args) : $message), \E_USER_DEPRECATED);
}
}"#,
    )
    .expect("conditional typed variadic function should parse");
}

/// Verifies comparison operators parse with lower precedence than arithmetic.
#[test]
fn parse_fragment_accepts_comparison_source() {
    let program = parse_fragment(br#"return $i + 1 < 3;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::Lt,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Add,
                left: Box::new(EvalExpr::LoadVar("i".to_string())),
                right: Box::new(EvalExpr::Const(EvalConst::Int(1))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
        }))]
    );
}
/// Verifies the spaceship operator parses at ordered-comparison precedence.
#[test]
fn parse_fragment_accepts_spaceship_source() {
    let program = parse_fragment(br#"return $i + 1 <=> 3;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::Spaceship,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Add,
                left: Box::new(EvalExpr::LoadVar("i".to_string())),
                right: Box::new(EvalExpr::Const(EvalConst::Int(1))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
        }))]
    );
}
/// Verifies loose equality operators parse as binary EvalIR expressions.
#[test]
fn parse_fragment_accepts_loose_equality_source() {
    let program = parse_fragment(br#"return "a" != "b";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LooseNotEq,
            left: Box::new(EvalExpr::Const(EvalConst::String("a".to_string()))),
            right: Box::new(EvalExpr::Const(EvalConst::String("b".to_string()))),
        }))]
    );
}
/// Verifies strict equality operators parse as distinct EvalIR comparisons.
#[test]
fn parse_fragment_accepts_strict_equality_source() {
    let program =
        parse_fragment(br#"return "10" === "10" && "10" !== 10;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LogicalAnd,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::StrictEq,
                left: Box::new(EvalExpr::Const(EvalConst::String("10".to_string()))),
                right: Box::new(EvalExpr::Const(EvalConst::String("10".to_string()))),
            }),
            right: Box::new(EvalExpr::Binary {
                op: EvalBinOp::StrictNotEq,
                left: Box::new(EvalExpr::Const(EvalConst::String("10".to_string()))),
                right: Box::new(EvalExpr::Const(EvalConst::Int(10))),
            }),
        }))]
    );
}
/// Verifies static `instanceof` parses as a high-precedence EvalIR expression.
#[test]
fn parse_fragment_accepts_static_instanceof_source() {
    let program =
        parse_fragment(br#"return !$object instanceof App\Box;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Unary {
            op: EvalUnaryOp::LogicalNot,
            expr: Box::new(EvalExpr::InstanceOf {
                value: Box::new(EvalExpr::LoadVar("object".to_string())),
                target: EvalInstanceOfTarget::ClassName("App\\Box".to_string()),
            }),
        }))]
    );
}

/// Verifies dynamic `instanceof` targets parse from variables, properties, arrays, and parens.
#[test]
fn parse_fragment_accepts_dynamic_instanceof_targets() {
    let program = parse_fragment(
        br#"return $object instanceof $names[0] . ":" . ($object instanceof ($prefix . $suffix));"#,
    )
    .expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::Concat,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Concat,
                left: Box::new(EvalExpr::InstanceOf {
                    value: Box::new(EvalExpr::LoadVar("object".to_string())),
                    target: EvalInstanceOfTarget::Expr(Box::new(EvalExpr::ArrayGet {
                        array: Box::new(EvalExpr::LoadVar("names".to_string())),
                        index: Box::new(EvalExpr::Const(EvalConst::Int(0))),
                    })),
                }),
                right: Box::new(EvalExpr::Const(EvalConst::String(":".to_string()))),
            }),
            right: Box::new(EvalExpr::InstanceOf {
                value: Box::new(EvalExpr::LoadVar("object".to_string())),
                target: EvalInstanceOfTarget::Expr(Box::new(EvalExpr::Binary {
                    op: EvalBinOp::Concat,
                    left: Box::new(EvalExpr::LoadVar("prefix".to_string())),
                    right: Box::new(EvalExpr::LoadVar("suffix".to_string())),
                })),
            }),
        }))]
    );
}

/// Verifies scalar cast syntax parses with PHP cast precedence across concatenation.
#[test]
fn parse_fragment_accepts_scalar_cast_source() {
    let program =
        parse_fragment(br#"return (string)$value . "!";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Cast {
            target: EvalCastType::String,
            expr: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Concat,
                left: Box::new(EvalExpr::LoadVar("value".to_string())),
                right: Box::new(EvalExpr::Const(EvalConst::String("!".to_string()))),
            }),
        }))]
    );
}

/// Verifies `(array)` casts lower through the dynamic cast expression node.
#[test]
fn parse_fragment_accepts_array_cast_source() {
    let program = parse_fragment(br#"return (array) $value;"#).expect("array cast should parse");
    assert!(matches!(
        program.statements(),
        [EvalStmt::Return(Some(EvalExpr::Cast {
            target: EvalCastType::Array,
            expr,
        }))] if matches!(expr.as_ref(), EvalExpr::LoadVar(name) if name == "value")
    ));
}

/// Verifies logical operators parse with `&&` binding tighter than `||`.
#[test]
fn parse_fragment_accepts_short_circuit_logical_source() {
    let program = parse_fragment(br#"return $a && $b || false;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LogicalOr,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::LogicalAnd,
                left: Box::new(EvalExpr::LoadVar("a".to_string())),
                right: Box::new(EvalExpr::LoadVar("b".to_string())),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Bool(false))),
        }))]
    );
}
/// Verifies PHP logical keywords parse case-insensitively with their own precedence.
#[test]
fn parse_fragment_accepts_keyword_logical_source() {
    let program =
        parse_fragment(br#"return false || true AnD false;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LogicalAnd,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::LogicalOr,
                left: Box::new(EvalExpr::Const(EvalConst::Bool(false))),
                right: Box::new(EvalExpr::Const(EvalConst::Bool(true))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Bool(false))),
        }))]
    );
}
/// Verifies PHP `xor` binds between `or` and `and` in eval expressions.
#[test]
fn parse_fragment_accepts_keyword_xor_source() {
    let program =
        parse_fragment(br#"return true XoR false or false;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LogicalOr,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::LogicalXor,
                left: Box::new(EvalExpr::Const(EvalConst::Bool(true))),
                right: Box::new(EvalExpr::Const(EvalConst::Bool(false))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Bool(false))),
        }))]
    );
}
/// Verifies ternary expressions parse below logical OR and preserve both branches.
#[test]
fn parse_fragment_accepts_ternary_source() {
    let program =
        parse_fragment(br#"return $a || $b ? "yes" : "no";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Ternary {
            condition: Box::new(EvalExpr::Binary {
                op: EvalBinOp::LogicalOr,
                left: Box::new(EvalExpr::LoadVar("a".to_string())),
                right: Box::new(EvalExpr::LoadVar("b".to_string())),
            }),
            then_branch: Some(Box::new(EvalExpr::Const(EvalConst::String(
                "yes".to_string()
            )))),
            else_branch: Box::new(EvalExpr::Const(EvalConst::String("no".to_string()))),
        }))]
    );
}
/// Verifies PHP's short ternary form omits the explicit then branch in EvalIR.
#[test]
fn parse_fragment_accepts_short_ternary_source() {
    let program = parse_fragment(br#"return $name ?: "fallback";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Ternary {
            condition: Box::new(EvalExpr::LoadVar("name".to_string())),
            then_branch: None,
            else_branch: Box::new(EvalExpr::Const(EvalConst::String("fallback".to_string()))),
        }))]
    );
}
/// Verifies null coalescing parses as a right-associative expression.
#[test]
fn parse_fragment_accepts_null_coalesce_source() {
    let program =
        parse_fragment(br#"return $a ?? $b ?? "fallback";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::NullCoalesce {
            value: Box::new(EvalExpr::LoadVar("a".to_string())),
            default: Box::new(EvalExpr::NullCoalesce {
                value: Box::new(EvalExpr::LoadVar("b".to_string())),
                default: Box::new(EvalExpr::Const(EvalConst::String("fallback".to_string()))),
            }),
        }))]
    );
}
/// Verifies match expressions preserve subject, patterns, and default expression.
#[test]
fn parse_fragment_accepts_match_source() {
    let program = parse_fragment(br#"return match ($x) { 1, 2 => "small", default => "other" };"#)
        .expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Match {
            subject: Box::new(EvalExpr::LoadVar("x".to_string())),
            arms: vec![EvalMatchArm {
                patterns: vec![
                    EvalExpr::Const(EvalConst::Int(1)),
                    EvalExpr::Const(EvalConst::Int(2)),
                ],
                value: EvalExpr::Const(EvalConst::String("small".to_string())),
            }],
            default: Some(Box::new(EvalExpr::Const(EvalConst::String(
                "other".to_string()
            )))),
        }))]
    );
}
/// Verifies null coalescing binds tighter than PHP ternary expressions.
#[test]
fn parse_fragment_null_coalesce_binds_tighter_than_ternary() {
    let program =
        parse_fragment(br#"return $a ?? $b ? "yes" : "no";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Ternary {
            condition: Box::new(EvalExpr::NullCoalesce {
                value: Box::new(EvalExpr::LoadVar("a".to_string())),
                default: Box::new(EvalExpr::LoadVar("b".to_string())),
            }),
            then_branch: Some(Box::new(EvalExpr::Const(EvalConst::String(
                "yes".to_string()
            )))),
            else_branch: Box::new(EvalExpr::Const(EvalConst::String("no".to_string()))),
        }))]
    );
}
/// Verifies logical negation parses as a unary expression before comparisons.
#[test]
fn parse_fragment_accepts_logical_not_source() {
    let program = parse_fragment(br#"return !$flag == true;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::LooseEq,
            left: Box::new(EvalExpr::Unary {
                op: EvalUnaryOp::LogicalNot,
                expr: Box::new(EvalExpr::LoadVar("flag".to_string())),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Bool(true))),
        }))]
    );
}
/// Verifies unary numeric operators bind tighter than multiplication.
#[test]
fn parse_fragment_accepts_unary_numeric_source() {
    let program = parse_fragment(br#"return -$x * +2;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::Mul,
            left: Box::new(EvalExpr::Unary {
                op: EvalUnaryOp::Negate,
                expr: Box::new(EvalExpr::LoadVar("x".to_string())),
            }),
            right: Box::new(EvalExpr::Unary {
                op: EvalUnaryOp::Plus,
                expr: Box::new(EvalExpr::Const(EvalConst::Int(2))),
            }),
        }))]
    );
}
