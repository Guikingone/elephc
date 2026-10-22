//! Purpose:
//! Pins the ownership of string offset results before their receiver is retired.
//!
//! Called from:
//! - The AST-to-EIR unit test module.
//!
//! Key details:
//! - Exercises the shared lowering and assembly generation for every supported target.

/// Review regressions preserve transient receiver releases and retire keys before suspension.
#[test]
fn string_offset_review_boundaries_on_all_targets() {
    use crate::ir::{Op, Ownership};
    let source = r#"<?php
function reviewConcatByte(int $index): string { return (str_repeat("a", 70000) . "b")[$index]; }
function reviewKey(array<mixed> &$items, string $bytes): void { $items[$bytes[0]] = 1; }
function reviewYield(string $bytes) { yield $bytes[0] => 1; }
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source, std::path::Path::new("main.php"), std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        let yielded = module.functions.iter().find(|function| function.name == "reviewYield").unwrap();
        let suspend = yielded.instructions.iter().position(|inst| inst.op == Op::GeneratorYield).unwrap();
        let key = yielded.instructions[suspend].operands[0];
        assert_eq!(yielded.values[key.as_raw() as usize].ownership, Ownership::Owned, "{target}");
        assert!(yielded.instructions[..suspend].iter().any(|inst| inst.op == Op::MixedBox), "{target}");
        assert!(yielded.instructions[..suspend].iter().any(|inst| inst.op == Op::PopCallOperandOwner), "{target}");
        assert!(yielded.instructions[..suspend].iter().any(|inst| inst.op == Op::ReleaseLocalSlot), "{target}");
        let asm = crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
        assert!(asm.contains("release_transient_skip"), "{target}: release unadopted concat storage");
    }
}

/// An offset view is copied before an owned receiver can be released on every target.
#[test]
fn string_offset_lifetime_stabilizes_views_on_all_targets() {
    use crate::ir::{Immediate, Op};
    let source = r#"<?php
class OffsetReader {
    public string $bytes;
    public function __construct(string $s) { $this->bytes = $s; }
    public function at(int $i): int { return ord($this->bytes[$i]); }
    public function emptyAt(int $i): bool { return empty($this->bytes[$i]); }
}
class OffsetStore {
    public string $typed = "";
    public $loose = "";
}
function offsetArray(array $items, int $i): string { return $items[0][$i]; }
function replaceOffsetBytes(OffsetReader $reader): int {
    $reader->bytes = "0";
    return 0;
}
function offsetStores(string $bytes): void {
    $local = $bytes . "!";
    $items = [""];
    $appended = [];
    $mixed = [1, ""];
    $hash = ["value" => ""];
    $keyed = [];
    $object = new OffsetStore();
    $items[0] = $local[8];
    $appended[] = $local[8];
    $mixed[0] = $local[8];
    $hash["value"] = $local[8];
    $keyed[$local[8]] = 1;
    $object->typed = $local[8];
    $object->loose = $local[8];
}
function offsetAcrossEffects(string $bytes, bool $select): void {
    $local = $bytes . "!";
    $keyed = [];
    $keyed[$local[8]] = ($local = "replaced");
    $local = $bytes . "!";
    echo $local[8] . ($local = "replaced");
    $local = $bytes . "!";
    echo $local[8] === ($local = "Z");
    $local = $bytes . "!";
    $joined = $select ? $local[8] : 88;
    $items = [];
    $items[] = $joined;
    $literalReader = new OffsetReader($bytes . "!");
    $literal = [$literalReader->bytes[8] => replaceOffsetBytes($literalReader)];
    $mixedLiteralReader = new OffsetReader($bytes . "!");
    $mixedLiteral = [0, $mixedLiteralReader->bytes[8] => replaceOffsetBytes($mixedLiteralReader)];
    $local = $bytes . "!";
    echo match ($local[8]) { ($local = "Z") => "yes", default => "no" };
    $reader = new OffsetReader($bytes . "!");
    echo $reader->bytes[8][replaceOffsetBytes($reader)];
}
class OffsetStaticStore {
    public static stdClass $object;
    public static mixed $kept;
    public static string $typed = "";
    public static $inferred = "";
    public static mixed $nested = ["seed" => ["child" => 0]];
}
function offsetCallableAfterChange(string $unused, callable $callback): callable { return $callback; }
function offsetYield(string $bytes) {
    $local = $bytes . "!";
    yield $local[8];
    $local = $bytes . "!";
    yield $local[8] => ($local = "no");
}
class OffsetAliasReturnReader {
    public function same(string $byte): string { return $byte; }
    public static function staticSame(string $byte): string { return $byte; }
}
#[AllowDynamicProperties]
class OffsetDynamicByteBag {}
function offsetAliasIdentity(string $byte): string { return $byte; }
function offsetAliasReturns(string $bytes, ?OffsetAliasReturnReader $nullable): void {
    $reader = new OffsetAliasReturnReader();
    $local = $bytes . "!";
    $kept = offsetAliasIdentity($local[8]);
    $local = "no";
    echo $kept;
    $local = $bytes . "!";
    $kept = $reader->same($local[8]);
    $local = "no";
    echo $kept;
    $local = $bytes . "!";
    $kept = OffsetAliasReturnReader::staticSame($local[8]);
    $local = "no";
    echo $kept;
    $local = $bytes . "!";
    $kept = $nullable?->same($local[8]);
    $local = "no";
    echo $kept;
    $local = $bytes . "!";
    $kept = strval($local[8]);
    $local = "no";
    echo $kept;
}
function offsetCallableAliasReturns(string $bytes): void {
    $local = $bytes . "!";
    $kept = (["OffsetAliasReturnReader", "staticSame"])($local[8]);
    $local = "no";
    echo $kept;
}
function offsetDynamicPropertyValues(string $bytes): void {
    $local = $bytes . "!";
    $object = new stdClass();
    $object->kept = $local[8];
    $local = $bytes . "!";
    $bag = new OffsetDynamicByteBag();
    $bag->kept = $local[8];
    $local = $bytes . "!";
    $name = "kept";
    $object->{$name} = $local[8];
}
function offsetReflectionStaticValues(string $bytes): void {
    $local = $bytes . "!";
    $class = new ReflectionClass(OffsetStaticStore::class);
    $class->setStaticPropertyValue("kept", $local[8]);
    $local = $bytes . "!";
    $property = new ReflectionProperty(OffsetStaticStore::class, "kept");
    $property->setValue(null, $local[8]);
    $local = $bytes . "!";
    $class->setStaticPropertyValue("typed", $local[8]);
    $local = $bytes . "!";
    $typed = new ReflectionProperty(OffsetStaticStore::class, "typed");
    $typed->setValue(null, $local[8]);
    $local = $bytes . "!";
    $class->setStaticPropertyValue("inferred", $local[8]);
    $local = $bytes . "!";
    $inferred = new ReflectionProperty(OffsetStaticStore::class, "inferred");
    $inferred->setValue(null, $local[8]);
}
function offsetNestedUpdateKey(string $bytes): void {
    $local = $bytes . "!";
    OffsetStaticStore::$nested[$local[8]]["child"]++;
}
function offsetBoundaries(string $bytes): void {
    $local = $bytes . "!";
    switch ($local[8]) {
        case ($local = "no"): echo "bad"; break;
        case "other": echo "bad"; break;
        default: echo "default";
    }
    $local = $bytes . "!";
    echo $local[8] |> (function (string $c): int { echo ord($c), ":"; return 1; });
    $callback = function (string $c): int { echo ord($c), ":"; return 1; };
    $local = $bytes . "!";
    echo $local[8] |> offsetCallableAfterChange($local = "no", $callback);
    $object = new stdClass();
    $local = $bytes . "!";
    $object->{$local[8]} = 1;
    $local = $bytes . "!";
    $object->{$local[8]} = ($local = "no");
    OffsetStaticStore::$object = new stdClass();
    $local = $bytes . "!";
    OffsetStaticStore::$object->{$local[8]} = ($local = "no");
    foreach (offsetYield($bytes) as $key => $value) { echo $key, $value; }
}
echo (new OffsetReader("hello"))->at($argc), offsetArray(["world"], $argc),
    (new OffsetReader("0a"))->emptyAt($argc);
offsetStores("\x07aaaaaaaZ");
offsetAcrossEffects("\x07aaaaaaaZ", $argc >= 0);
offsetAliasReturns("\x07aaaaaaaZ", new OffsetAliasReturnReader());
offsetCallableAliasReturns("\x07aaaaaaaZ");
offsetDynamicPropertyValues("\x07aaaaaaaZ");
offsetReflectionStaticValues("\x07aaaaaaaZ");
offsetNestedUpdateKey("\x07aaaaaaaZ");
offsetBoundaries("\x07aaaaaaaZ");
"#;
    for target in ["macos-aarch64", "ios-arm64", "ios-sim-arm64", "linux-aarch64", "linux-x86_64"] {
        let module = super::lower_source_at_for_target(
            source, std::path::Path::new("main.php"), std::path::Path::new("."),
            crate::codegen::platform::Target::parse(target).unwrap(),
        );
        let mut reads = 0;
        let mut store_reads = 0;
        let mut persisted_reads = 0;
        for function in module.functions.iter().chain(module.class_methods.iter()) {
            if function.name == "offsetNestedUpdateKey" {
                let byte_results = function.instructions.iter()
                    .filter(|inst| inst.op == Op::StrCharAt)
                    .filter_map(|inst| inst.result)
                    .collect::<Vec<_>>();
                assert_eq!(byte_results.len(), 2,
                    "{target}: the static nested update reads and writes the parent key");
                let stabilized = byte_results.iter().filter(|result| {
                    function.instructions.iter().any(|inst| {
                        inst.op == Op::Acquire && inst.operands == [**result]
                    })
                }).count();
                assert_eq!(stabilized, 1,
                    "{target}: persist only the nested write key that can reach hash insertion");
                let pinned = byte_results.iter().filter(|result| {
                    let mut acquired = vec![**result];
                    for inst in &function.instructions {
                        if inst.op == Op::Acquire && inst.operands.iter().any(|value| acquired.contains(value)) {
                            if let Some(value) = inst.result { acquired.push(value); }
                        }
                    }
                    if acquired.len() == 1 { return false; }
                    let slots = function.instructions.iter().filter_map(|inst| {
                        (inst.op == Op::StoreLocal
                            && inst.operands.iter().any(|value| acquired.contains(value)))
                            .then_some(inst.immediate.as_ref())
                            .flatten()
                            .and_then(|immediate| match immediate {
                                Immediate::LocalSlot(slot) => Some(*slot),
                                _ => None,
                            })
                    });
                    slots.into_iter().any(|slot| {
                        function.instructions.iter().any(|inst| {
                            inst.op == Op::PushCallOperandOwner
                                && inst.immediate == Some(Immediate::LocalSlot(slot))
                        }) && function.instructions.iter().any(|inst| {
                            inst.op == Op::PopCallOperandOwner
                                && inst.immediate == Some(Immediate::LocalSlot(slot))
                        })
                    })
                }).count();
                assert_eq!(pinned, 1,
                    "{target}: publish the persisted nested key through every throwing writer");
                reads += byte_results.len();
                persisted_reads += stabilized;
                continue;
            }
            if function.name == "offsetBoundaries" {
                assert!(function.blocks.iter().any(|block| block.name == "switch.matched"),
                    "{target}: exercise dynamic switch dispatch without single-case normalization");
            }
            for (index, read) in function.instructions.iter().enumerate() {
                if read.op != Op::StrCharAt { continue; }
                reads += 1;
                if function.name == "offsetStores" {
                    store_reads += 1;
                    persisted_reads += 1;
                    continue;
                }
                if matches!(
                    function.name.as_str(),
                    "offsetAcrossEffects" | "offsetAliasReturns" | "offsetBoundaries"
                ) {
                    assert!(function.instructions[index + 1..].iter().any(|inst| {
                        matches!(inst.op, Op::Acquire | Op::MixedBox)
                            && inst.operands.contains(&read.result.unwrap())
                    }), "{target}: {} byte at {:?} must survive the following effect", function.name, read.span);
                    persisted_reads += 1;
                    continue;
                }
                if function.name == "offsetYield" {
                    assert!(function.instructions[index + 1..].iter().any(|inst| {
                        matches!(inst.op, Op::Acquire | Op::GeneratorYield)
                            && inst.operands.contains(&read.result.unwrap())
                    }), "{target}: preserve yielded bytes and explicit keys");
                    persisted_reads += 1;
                    continue;
                }
                if function.name == "offsetCallableAliasReturns" {
                    assert!(function.instructions[index + 1..].iter().any(|inst| {
                        inst.op == Op::ExprCall && inst.operands.contains(&read.result.unwrap())
                    }), "{target}: descriptor argument must preserve the direct byte producer");
                    persisted_reads += 1;
                    continue;
                }
                if function.name == "offsetDynamicPropertyValues" {
                    let stored = function.instructions[index + 1..].iter().any(|inst| {
                        matches!(inst.op, Op::PropSet | Op::DynamicPropSet)
                            && inst.operands.contains(&read.result.unwrap())
                    });
                    if stored {
                        persisted_reads += 1;
                    } else {
                        assert!(function.instructions[index + 1..].iter().any(|inst| {
                            inst.op == Op::Release && inst.operands.contains(&read.result.unwrap())
                        }), "{target}: duplicated dynamic-write byte reads must only feed cleanup");
                    }
                    continue;
                }
                if function.name == "offsetReflectionStaticValues" {
                    assert!(function.instructions[index + 1..].iter().any(|inst| {
                        inst.op == Op::StoreReflectionStaticProperty
                            && inst.operands.contains(&read.result.unwrap())
                    }), "{target}: reflection static stores must preserve the direct byte producer");
                    persisted_reads += 1;
                    continue;
                }
                if function.name.ends_with("::emptyAt") {
                    // Silent empty stages the receiver in a local that outlives the boolean test.
                    assert!(function.instructions.iter().any(|inst| inst.op == Op::IsTruthy));
                    continue;
                }
                let copied = function.instructions[index + 1..].iter().position(|inst| {
                    inst.op == Op::Acquire && inst.operands == [read.result.unwrap()]
                }).expect("an offset view needs an independent byte owner");
                let released = function.instructions[index + 1..].iter().position(|inst| {
                    inst.op == Op::Release && inst.operands == [read.operands[0]]
                }).expect("the owning receiver must be retired");
                assert!(copied < released, "{target}: copy the byte before freeing its source");
                persisted_reads += 1;
            }
        }
        assert_eq!(reads, 44, "{target}: reads and persistent stores exercise StrCharAt");
        assert_eq!(store_reads, 7,
            "{target}: cover indexed, mixed, hash-key, hash-value and property stores");
        let asm = crate::codegen::generate_user_asm_from_ir(&module, false, false).unwrap();
        assert!(asm.contains("str_byte_persist_ready"), "{target}: byte persistence must bypass heap header detection");
        let stage = if target == "linux-x86_64" { "mov BYTE PTR [rsp], r10b" } else { "strb w9, [sp]" };
        assert!(asm.matches(stage).count() >= persisted_reads,
            "{target}: copy every persisted byte to unmanaged stack storage");
    }
}
