//! Purpose:
//! Parser tests for assignment, compound assignment, increment/decrement, and echo statements.
//!
//! Called from:
//! - `cargo test -p elephc-magician` through Rust's test harness.
//!
//! Key details:
//! - These cases assert direct statement lowering into EvalIR stores and echoes.

use super::support::*;

/// Verifies assignment fragments lower to by-name StoreVar statements.
#[test]
fn parse_fragment_accepts_assignment_source() {
    let program = parse_fragment(b"$x = 1;").expect("fragment should parse");
    assert_eq!(program.source_len(), 7);
    assert_eq!(
        program.statements(),
        &[EvalStmt::StoreVar {
            name: "x".to_string(),
            value: EvalExpr::Const(EvalConst::Int(1)),
        }]
    );
}

/// Verifies null-coalescing assignment is an expression with a writable array target.
#[test]
fn parse_fragment_accepts_null_coalesce_assignment_expression() {
    let program = parse_fragment(br#"return $_SERVER["option"] ??= [];"#)
        .expect("null-coalescing assignment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::NullCoalesceAssign {
            target: Box::new(EvalExpr::ArrayGet {
                array: Box::new(EvalExpr::LoadVar("_SERVER".to_string())),
                index: Box::new(EvalExpr::Const(EvalConst::String("option".to_string()))),
            }),
            default: Box::new(EvalExpr::Array(Vec::new())),
        }))]
    );
}

/// Verifies null-coalescing assignment remains right-associative for variable targets.
#[test]
fn parse_fragment_keeps_null_coalesce_assignment_right_associative() {
    let program =
        parse_fragment(b"return $left ??= $right ??= 7;").expect("assignment chain should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::NullCoalesceAssign {
            target: Box::new(EvalExpr::LoadVar("left".to_string())),
            default: Box::new(EvalExpr::NullCoalesceAssign {
                target: Box::new(EvalExpr::LoadVar("right".to_string())),
                default: Box::new(EvalExpr::Const(EvalConst::Int(7))),
            }),
        }))]
    );
}

/// Verifies a statement-level nested array target remains one writable `??=` expression.
#[test]
fn parse_fragment_accepts_nested_array_null_coalesce_assignment() {
    let program = parse_fragment(br#"$items["outer"]["inner"] ??= 4;"#)
        .expect("nested null-coalescing assignment should parse");
    assert!(matches!(
        program.statements(),
        [EvalStmt::Expr(EvalExpr::NullCoalesceAssign { target, .. })]
            if matches!(target.as_ref(), EvalExpr::ArrayGet { array, .. }
                if matches!(array.as_ref(), EvalExpr::ArrayGet { .. }))
    ));
}

/// Verifies a static-property store can contain a right-associative variable assignment.
#[test]
fn parse_fragment_accepts_chained_assignment_expression() {
    let program = parse_fragment(br#"self::$loader = $loader = new \stdClass();"#)
        .expect("chained assignment should parse");
    assert!(matches!(
        program.statements(),
        [EvalStmt::StaticPropertySet {
            value: EvalExpr::Assign { target, .. },
            ..
        }] if matches!(target.as_ref(), EvalExpr::LoadVar(name) if name == "loader")
    ));
}

/// Verifies a generated runtime bootstrap shape parses as one complete fragment.
#[test]
fn parse_fragment_accepts_generated_runtime_bootstrap_shape() {
    parse_fragment(
        br#"
if (true === (require_once __DIR__.'/autoload.php') || empty($_SERVER['SCRIPT_FILENAME'])) {
    return;
}
$app = require $_SERVER['SCRIPT_FILENAME'];
if (!is_object($app)) {
    throw new TypeError(sprintf('invalid %s from %s', get_debug_type($app), $_SERVER['SCRIPT_FILENAME']));
}
if (is_string($_SERVER['APP_RUNTIME_OPTIONS'] ??= $_ENV['APP_RUNTIME_OPTIONS'] ?? [])) {
    $_SERVER['APP_RUNTIME_OPTIONS'] = json_decode($_SERVER['APP_RUNTIME_OPTIONS'], true, 512, JSON_THROW_ON_ERROR);
}
$_SERVER['APP_RUNTIME'] ??= $_ENV['APP_RUNTIME'] ?? 'Runtime\\DefaultRuntime';
$runtime = new $_SERVER['APP_RUNTIME']($_SERVER['APP_RUNTIME_OPTIONS'] += [
    'project_dir' => dirname(__DIR__, 1),
]);
[$app, $args] = $runtime->getResolver($app)->resolve();
$app = $app(...$args);
exit($runtime->getRunner($app)->run());
"#,
    )
    .expect("generated runtime bootstrap should parse");
}

/// Verifies a generated autoloader initialization shape parses completely.
#[test]
fn parse_fragment_accepts_generated_autoloader_shape() {
    parse_fragment(
        br#"
class GeneratedAutoloaderInitFixture
{
    private static $loader;
    public static function loadClassLoader($class)
    {
        if ('Vendor\Autoload\ClassLoader' === $class) {
            require __DIR__ . '/ClassLoader.php';
        }
    }
    public static function getLoader()
    {
        if (null !== self::$loader) {
            return self::$loader;
        }
        require __DIR__ . '/platform_check.php';
        spl_autoload_register(array('GeneratedAutoloaderInitFixture', 'loadClassLoader'), true, true);
        self::$loader = $loader = new \Vendor\Autoload\ClassLoader(\dirname(__DIR__));
        spl_autoload_unregister(array('GeneratedAutoloaderInitFixture', 'loadClassLoader'));
        require __DIR__ . '/autoload_static.php';
        call_user_func(\Vendor\Autoload\StaticInitFixture::getInitializer($loader));
        $loader->register(true);
        $filesToLoad = \Vendor\Autoload\StaticInitFixture::$files;
        $requireFile = \Closure::bind(static function ($fileIdentifier, $file) {
            if (empty($GLOBALS['__autoload_files'][$fileIdentifier])) {
                $GLOBALS['__autoload_files'][$fileIdentifier] = true;
                require $file;
            }
        }, null, null);
        foreach ($filesToLoad as $fileIdentifier => $file) {
            $requireFile($fileIdentifier, $file);
        }
        return $loader;
    }
}
"#,
    )
    .expect("generated autoloader shape should parse");
}

/// Verifies reference assignments lower to by-name ReferenceAssign statements.
#[test]
fn parse_fragment_accepts_reference_assignment_source() {
    let program = parse_fragment(b"$left =& $right;").expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::ReferenceAssign {
            target: "left".to_string(),
            source: "right".to_string(),
        }]
    );
}
/// Verifies multiplicative operators preserve PHP precedence and associativity.
#[test]
fn parse_fragment_accepts_division_and_modulo_source() {
    let program = parse_fragment(b"return 10 / 4 % 3;").expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::Mod,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Div,
                left: Box::new(EvalExpr::Const(EvalConst::Int(10))),
                right: Box::new(EvalExpr::Const(EvalConst::Int(4))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
        }))]
    );
}
/// Verifies exponentiation is right-associative and binds tighter than unary negation.
#[test]
fn parse_fragment_accepts_power_source() {
    let program =
        parse_fragment(b"return -2 ** 2; return 2 ** 3 ** 2;").expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[
            EvalStmt::Return(Some(EvalExpr::Unary {
                op: EvalUnaryOp::Negate,
                expr: Box::new(EvalExpr::Binary {
                    op: EvalBinOp::Pow,
                    left: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                }),
            })),
            EvalStmt::Return(Some(EvalExpr::Binary {
                op: EvalBinOp::Pow,
                left: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                right: Box::new(EvalExpr::Binary {
                    op: EvalBinOp::Pow,
                    left: Box::new(EvalExpr::Const(EvalConst::Int(3))),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                }),
            })),
        ]
    );
}
/// Verifies bitwise operators preserve PHP precedence.
#[test]
fn parse_fragment_accepts_bitwise_source() {
    let program = parse_fragment(b"return ~0 | 2 ^ 3 & 4;").expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::BitOr,
            left: Box::new(EvalExpr::Unary {
                op: EvalUnaryOp::BitNot,
                expr: Box::new(EvalExpr::Const(EvalConst::Int(0))),
            }),
            right: Box::new(EvalExpr::Binary {
                op: EvalBinOp::BitXor,
                left: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                right: Box::new(EvalExpr::Binary {
                    op: EvalBinOp::BitAnd,
                    left: Box::new(EvalExpr::Const(EvalConst::Int(3))),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(4))),
                }),
            }),
        }))]
    );
}
/// Verifies shift operators bind lower than additive expressions.
#[test]
fn parse_fragment_accepts_shift_source() {
    let program = parse_fragment(b"return 1 + 2 << 3;").expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Return(Some(EvalExpr::Binary {
            op: EvalBinOp::ShiftLeft,
            left: Box::new(EvalExpr::Binary {
                op: EvalBinOp::Add,
                left: Box::new(EvalExpr::Const(EvalConst::Int(1))),
                right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
            }),
            right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
        }))]
    );
}
/// Verifies simple variable compound assignments lower to StoreVar with binary expressions.
#[test]
fn parse_fragment_accepts_compound_assignment_source() {
    let program = parse_fragment(br#"$x += 2; $x -= 1; $x *= 3; $x /= 2; $x %= 5; $s .= "ok";"#)
        .expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Add,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Sub,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(1))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Mul,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Div,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Mod,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(5))),
                },
            },
            EvalStmt::StoreVar {
                name: "s".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::Concat,
                    left: Box::new(EvalExpr::LoadVar("s".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::String("ok".to_string()))),
                },
            },
        ]
    );
}
/// Verifies exponentiation compound assignment lowers through the binary power operator.
#[test]
fn parse_fragment_accepts_power_compound_assignment_source() {
    let program = parse_fragment(br#"$x **= 3;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::StoreVar {
            name: "x".to_string(),
            value: EvalExpr::Binary {
                op: EvalBinOp::Pow,
                left: Box::new(EvalExpr::LoadVar("x".to_string())),
                right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
            },
        }]
    );
}
/// Verifies bitwise compound assignments lower to StoreVar with binary expressions.
#[test]
fn parse_fragment_accepts_bitwise_compound_assignment_source() {
    let program = parse_fragment(br#"$x &= 3; $x |= 1; $x ^= 2; $x <<= 4; $x >>= 1;"#)
        .expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::BitAnd,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(3))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::BitOr,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(1))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::BitXor,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(2))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::ShiftLeft,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(4))),
                },
            },
            EvalStmt::StoreVar {
                name: "x".to_string(),
                value: EvalExpr::Binary {
                    op: EvalBinOp::ShiftRight,
                    left: Box::new(EvalExpr::LoadVar("x".to_string())),
                    right: Box::new(EvalExpr::Const(EvalConst::Int(1))),
                },
            },
        ]
    );
}
/// Verifies simple variable increment and decrement statements lower to StoreVar.
#[test]
fn parse_fragment_accepts_inc_dec_statement_source() {
    let program = parse_fragment(br#"$i++; ++$j; $k--; --$m;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[
            inc_dec_store("i".to_string(), true),
            inc_dec_store("j".to_string(), true),
            inc_dec_store("k".to_string(), false),
            inc_dec_store("m".to_string(), false),
        ]
    );
}
/// Verifies echo fragments preserve expression source order.
#[test]
fn parse_fragment_accepts_echo_source() {
    let program = parse_fragment(br#"echo "hi" . $name;"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[EvalStmt::Echo(EvalExpr::Binary {
            op: EvalBinOp::Concat,
            left: Box::new(EvalExpr::Const(EvalConst::String("hi".to_string()))),
            right: Box::new(EvalExpr::LoadVar("name".to_string())),
        })]
    );
}
/// Verifies PHP echo comma lists lower to one EvalIR echo statement per expression.
#[test]
fn parse_fragment_accepts_echo_comma_list_source() {
    let program = parse_fragment(br#"echo "a", $b, "c";"#).expect("fragment should parse");
    assert_eq!(
        program.statements(),
        &[
            EvalStmt::Echo(EvalExpr::Const(EvalConst::String("a".to_string()))),
            EvalStmt::Echo(EvalExpr::LoadVar("b".to_string())),
            EvalStmt::Echo(EvalExpr::Const(EvalConst::String("c".to_string()))),
        ]
    );
}
