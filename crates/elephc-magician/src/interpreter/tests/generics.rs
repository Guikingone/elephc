//! Purpose:
//! Runtime regression tests for native generic templates and typed contracts.
//!
//! Called from:
//! - `cargo test -p elephc-magician generics`.
//!
//! Key details:
//! - Persistent contexts exercise declarations and specialization across separate fragments.

use super::support::*;
use super::super::*;

/// Executes fragments in one context and returns captured output.
fn run(fragments: &[&str]) -> String {
    let mut context = ElephcEvalContext::new();
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();
    for source in fragments {
        let program = parse_fragment(source.as_bytes()).expect("generic syntax parses");
        let result = execute_program_with_context(&mut context, &program, &mut scope, &mut values)
            .unwrap_or_else(|error| panic!("{source}: {error:?}; output: {}", values.output));
        values.release(result).expect("release eval result");
    }
    values.output
}

/// Functions specialize independently for scalars, nested arrays, names, and spreads.
#[test]
fn generics_functions_infer_and_persist() {
    assert_eq!(run(&[
        "function identity<T>(T $value): T { return $value; } function first<T>(array<T> $items): T { return $items[0]; }",
        "echo identity(41), '|', identity('ok'), '|', first([[7]])[0], '|', identity(...['value' => 'spread']);",
    ]), "41|ok|7|spread");
}

/// Explicit arguments and defaults use the same concrete signature as inferred calls.
#[test]
fn generics_written_and_dependent_defaults() {
    assert_eq!(run(&[
        "function same<T, U = T>(T $a, U $b): U { return $b; } echo same<int>(1, 2); function emptyItems<T>(): array<T> { return []; } echo count(emptyItems<string>());",
    ]), "20");
}

/// Written and constructor-inferred class instantiations keep distinct identities and properties.
#[test]
fn generics_classes_have_concrete_identity() {
    assert_eq!(run(&[
        "class Box<T> { public function __construct(private T $value) {} public function get(): T { return $this->value; } }",
        "$i = new Box<int>(41); $s = new Box('ok'); echo $i->get(), '|', $s->get(), '|', get_class($i), '|', get_class($s);",
    ]), "41|ok|Box<int>|Box<string>");
}

/// Conflicting, missing, and excess generic arguments fail as catchable TypeErrors.
#[test]
fn generics_binding_errors_are_catchable() {
    assert_eq!(run(&[
        "function pair<T>(T $a, T $b): T { return $a; } function missing<T>(int $a): int { return $a; }",
        "try { pair(1, 'x'); } catch (TypeError $e) { echo 'conflict|'; } try { missing(1); } catch (TypeError $e) { echo 'missing|'; } try { pair<int, string>(1, 2); } catch (TypeError $e) { echo 'excess'; }",
    ]), "conflict|missing|excess");
}

/// Bounds use the class/interface hierarchy rather than structural resemblance.
#[test]
fn generics_bounds_accept_subclasses() {
    assert_eq!(run(&[
        "class Base { public function id(): int { return 7; } } class Child extends Base {} function id<T: Base>(T $value): int { return $value->id(); } echo id(new Child()); try { id(1); } catch (TypeError $e) { echo '|bound'; }",
    ]), "7|bound");
}

/// Structural array contracts check all values on both argument and return boundaries.
#[test]
fn generics_typed_arrays_validate_elements() {
    assert_eq!(run(&[
        "function ints(array<int> $items): int { return count($items); } echo ints([1, 2]); try { ints(['x']); } catch (TypeError $e) { echo '|argument'; } function bad(): array<int> { return ['x']; } try { bad(); } catch (TypeError $e) { echo '|return'; }",
    ]), "2|argument|return");
}

/// Generic prototypes preserve type parameters in namespaced declarations.
#[test]
fn generics_namespace_imports_survive_specialization() {
    assert_eq!(run(&[
        "namespace Model; class Thing { public function get(): int { return 9; } }",
        "namespace Work; use Model\\Thing; function get<T: Thing>(T $value): int { return $value->get(); } echo get(new Thing());",
    ]), "9");
}

/// Runtime strict PHP mode rejects native generic syntax before any source statements execute.
#[test]
fn generics_native_syntax_rejected_in_strict_mode() {
    let _guard = crate::strict_php_mode::scoped_enable();
    let mut values = FakeOps::default();
    let mut scope = ElephcEvalScope::new();
    let program = parse_fragment(b"echo 'before'; function id<T>(T $v): T { return $v; }").unwrap();
    assert_eq!(execute_program(&program, &mut scope, &mut values), Err(EvalStatus::ParseError));
    assert_eq!(values.output, "");
}

/// A generic method has independent slots, including callable signature inference.
#[test]
fn generics_methods_and_typed_callables() {
    assert_eq!(run(&[
        "class Box<T> { public function __construct(private T $value) {} public function get(): T { return $this->value; } public function map<U>(callable(T): U $f): Box<U> { return new Box<U>($f($this->value)); } public static function id<V>(V $v): V { return $v; } }",
        "$b = new Box('hello'); echo $b->map(function(string $s): int { return strlen($s); })->get(), '|', $b->map<string>(function(string $s): string { return strtoupper($s); })->get(), '|', Box<int>::id<string>('ok');",
    ]), "5|HELLO|ok");
}

/// Portable template annotations specialize functions and member types in both PHP modes.
#[test]
fn generics_docblock_templates_in_strict_mode() {
    for strict in [false, true] {
        let _guard = strict.then(crate::strict_php_mode::scoped_enable);
        let output = run(&[
            "/** @template T */\nclass Box { /** @var T */ private $value; /** @param T $value */ public function __construct($value) { $this->value = $value; } /** @return T */ public function get() { return $this->value; } }\n/**\n * @template U\n * @param U $value\n * @return U\n */\nfunction identity($value) { return $value; }",
            "$b = new Box(7); echo $b->get(), '|', identity('ok');",
        ]);
        assert_eq!(output, "7|ok");
    }
}

/// Substitution visits type positions without renaming methods or constants named like a slot.
#[test]
fn generics_substitution_preserves_non_type_names() {
    assert_eq!(run(&[
        "class User { const T = 'constant'; public function T(): string { return 'method'; } } function label<T>(T $value): string { return $value->T() . User::T . 'T'; } echo label(new User());",
    ]), "methodconstantT");
}

/// Generic interfaces and traits are instantiated before implementing class validation.
#[test]
fn generics_interfaces_and_traits() {
    assert_eq!(run(&[
        "interface Getter<T> { public function get(): T; } trait Stored<T> { public T $value; public function get(): T { return $this->value; } } class IntBox implements Getter<int> { use Stored<int>; public function __construct(int $value) { $this->value = $value; } } echo (new IntBox(9))->get();",
    ]), "9");
}

/// Constructor-local parameters specialize promoted storage independently of class slots.
#[test]
fn generics_constructor_slots_and_shadowing() {
    assert_eq!(run(&[
        "class Item { public function __construct<T>(private T $value) {} public function get() { return $this->value; } } class Pair<T> { public function __construct<U>(private T $left, private U $right) {} public function right() { return $this->right; } } class Shadow<T> { public function __construct<T>(private T $value) {} public function get() { return $this->value; } }",
        "$a = new Item(7); $b = new Pair<int>(1, 'two'); $c = new Shadow<int>('shadow'); echo $a->get(), '|', get_class($a), '|', $b->right(), '|', get_class($b), '|', $c->get(), '|', get_class($c);",
    ]), "7|Item<int>|two|Pair<int><string>|shadow|Shadow<int><string>");
}

/// Imported generic function aliases retain explicitly written types in their own namespace.
#[test]
fn generics_function_aliases_and_enum_methods() {
    assert_eq!(run(&[
        "namespace Lib; function id<T>(T $v): T { return $v; }",
        "namespace App; use function Lib\\id as alias; echo alias<string>('ok');",
    ]), "ok");
    assert_eq!(run(&["enum Pick { case One; public function id<T>(T $v): T { return $v; } } echo Pick::One->id<int>(7);"]), "7");
}

/// Variance widens object arguments and keeps AOT's scalar storage restriction.
#[test]
fn generics_variance_accepts_safe_object_widening() {
    assert_eq!(run(&[
        "class Animal {} class Dog extends Animal {} class Box<out T> { public function __construct(private T $value) {} public function get(): T { return $this->value; } } class Sink<in T> { public function put(T $v): void {} } function read(Box<Animal> $box): string { return get_class($box->get()); } function write(Sink<Dog> $sink): string { $sink->put(new Dog()); return 'sink'; } echo read(new Box<Dog>(new Dog())), '|', write(new Sink<Animal>()); function mixedBox(Box<mixed> $box): void {} try { mixedBox(new Box<int>(7)); } catch (TypeError $e) { echo '|storage'; }",
    ]), "Dog|sink|storage");
}

/// Own, inherited, writable-property, and callable polarity violations are rejected.
#[test]
fn generics_variance_declarations_are_checked() {
    for declaration in [
        "class Bad<+T> { public function put(T $value): void {} }",
        "class Bad<+T> { public T $value; }",
        "class Bad<-T> { public function get(): T { return null; } }",
        "class Bad<+T> { public function get(): callable(T): int { return null; } }",
        "class ParentBox<T> { public function put(T $v): void {} } class Bad<+T> extends ParentBox<T> {}",
    ] {
        assert_eq!(run(&[&format!("try {{ {declaration} }} catch (TypeError $e) {{ echo 'variance'; }}")]), "variance");
    }
}

/// Unconditional generic declarations can be referenced before their source position.
#[test]
fn generics_forward_calls_are_registered() {
    assert_eq!(run(&["echo id(7); function id<T>(T $v): T { return $v; }"]), "7");
}

/// Substitution covers local types, nested closure signatures, and subsequent array writes.
#[test]
fn generics_local_and_closure_types_are_preserved() {
    assert_eq!(run(&[
        "function keep<T>(T $v): T { T $held = $v; $f = function(T $arg): T { return $arg; }; array<T> $items = [$held]; try { $held = 'wrong'; } catch (TypeError $e) { echo 'local|'; } try { $items[] = 'wrong'; } catch (TypeError $e) { echo 'append|'; } try { $items[0] = 'wrong'; } catch (TypeError $e) { echo 'index|'; } return $f($items[0]); } echo keep(7);",
    ]), "local|append|index|7");
}

/// Nullable evidence and constructor defaults do not infer from null or empty storage.
#[test]
fn generics_nullable_inference_and_empty_constructor_evidence() {
    assert_eq!(run(&[
        "function id<T>(?T $v): ?T { return $v; } echo id(7); class Box<T = string> { public function __construct(private ?T $v) {} } echo '|', get_class(new Box(null)); class Required<T> { public function __construct(T $v) {} } try { new Required(null); } catch (TypeError $e) { echo '|null'; } try { new Required([]); } catch (TypeError $e) { echo '|empty'; }",
    ]), "7|Box<string>|null|empty");
}

/// Serialized templates preserve lexical imports, type-use indexes, and strict provenance.
#[test]
fn generics_cached_templates_round_trip() {
    let program = parse_fragment(b"namespace App; class Box<T> { public function __construct(private T $v) {} public function get(): T { return $this->v; } } echo (new Box<int>(7))->get();").unwrap();
    let bytes = bincode::serialize(&program).unwrap();
    let loaded: EvalProgram = bincode::deserialize(&bytes).unwrap();
    assert_eq!(loaded, program);
    let mut values = FakeOps::default();
    let mut scope = ElephcEvalScope::new();
    let result = execute_program(&loaded, &mut scope, &mut values).unwrap();
    values.release(result).unwrap();
    assert_eq!(values.output, "7");
    let _guard = crate::strict_php_mode::scoped_enable();
    assert_eq!(execute_program(&loaded, &mut scope, &mut values), Err(EvalStatus::ParseError));
}

/// PHPDoc annotations respect attributes, native class slots, and ignored unsupported grammar.
#[test]
fn generics_docblocks_keep_attributes_and_native_class_scope() {
    assert_eq!(run(&[
        "class Box<T> { /** @param T $value */ public function __construct(private $value) {} /** @return T */ public function get() { return $this->value; } } echo (new Box<int>(7))->get(); /**\n * @template U\n * @param U $value\n * @return U\n */ #[Deprecated] function identity($value) { return $value; } echo '|', identity('ok'); /** @template X of unsupported[] */ function ordinary($v) { return $v; } echo '|', ordinary(8);",
    ]), "7|ok|8");
}

/// Bare type hints and bare inheritance use the declared bound before the default.
#[test]
fn generics_bare_types_and_inheritance_defaults() {
    assert_eq!(run(&[
        "class Animal {} class Dog extends Animal {} class Repo<out T: Animal = Dog> { public function __construct(private T $v) {} public function get(): T { return $this->v; } } function read(Repo $repo): string { return get_class($repo->get()); } interface Name<T = string> { public function name(): T; } class Named implements Name { public function name(): string { return 'named'; } } trait Store<T = int> { public T $v; } trait Nested { use Store; } class Holder { use Nested; } echo read(new Repo<Dog>(new Dog())), '|', (new Named())->name(); $h = new Holder(); $h->v = 7; echo '|', $h->v;",
    ]), "Dog|named|7");
}

/// Ordinary scalar types produced by specialization retain catchable contract failures.
#[test]
fn generics_specialized_scalar_contracts_throw_type_errors() {
    assert_eq!(run(&[
        "function id<T>(T $v): T { return $v; } function bad<T>(T $v): T { return []; } try { id<int>([]); } catch (TypeError $e) { echo 'argument|'; } try { bad(7); } catch (TypeError $e) { echo 'return'; }",
    ]), "argument|return");
}

/// Specialized methods retain the source spelling of magic constants and trait origins.
#[test]
fn generics_magic_constants_keep_source_names() {
    assert_eq!(run(&[
        "function label<T>(T $v): string { return __FUNCTION__; } trait Label { public function label<U>(U $v): string { return __TRAIT__ . ':' . __METHOD__; } } class Box<T> { use Label; public function __construct(T $v) {} public function source(): string { return __CLASS__ . ':' . __METHOD__; } } $b = new Box<int>(7); echo label(7), '|', $b->source(), '|', $b->label<string>('x');",
    ]), "label|Box:Box::source|Label:Label::label");
}

/// Trait adaptation qualifiers select the actual concrete trait, as they do in AOT.
#[test]
fn generics_trait_adaptations_select_specializations() {
    assert_eq!(run(&[
        "trait Label<T> { public function label(T $v): T { return $v; } } trait Other { public function label(int $v): int { return 0; } } class Box { use Label<int>, Other { Label::label insteadof Other; Label::label as renamed; } } $b = new Box(); echo $b->label(7), '|', $b->renamed(8);",
    ]), "7|8");
}

/// Recursive type growth terminates with a catchable error and leaves later calls usable.
#[test]
fn generics_recursive_growth_is_bounded() {
    assert_eq!(run(&[
        "function recurse<T>(T $v): void { recurse([$v]); } try { recurse(7); } catch (TypeError $e) { echo 'depth|'; } function id<T>(T $v): T { return $v; } echo id(8);",
    ]), "depth|8");
}

/// Constructor slots of an ordinary class may also be supplied explicitly.
#[test]
fn generics_written_constructor_slots_of_ordinary_class() {
    assert_eq!(run(&[
        "class Item { public function __construct<T>(public T $value) {} } $item = new Item<string>('ok'); echo $item->value, '|', get_class($item); try { new Item<int>([]); } catch (TypeError $e) { echo '|type'; }",
    ]), "ok|Item<string>|type");
}

/// Static writes create their selected class and static state remains separate per instantiation.
#[test]
fn generics_static_storage_is_per_instantiation() {
    assert_eq!(run(&[
        "class Counter<T> { public static int $made = 0; public function __construct(T $v) { static::$made++; } public static function of(T $v): Counter<T> { return new Counter<T>($v); } } Counter<int>::$made = 5; new Counter<int>(7); new Counter<string>('x'); echo Counter<int>::$made, '|', Counter<string>::$made, '|', get_class(Counter::of(8));",
    ]), "6|1|Counter<int>");
}

/// Native class slots take precedence over conflicting PHPDoc template declarations.
#[test]
fn generics_docblocks_preserve_native_precedence() {
    assert_eq!(run(&[
        "/** @template T */ class Box<U> { /** @var T */ public $ignored; /** @param U $value */ public function __construct(public $value) {} } $box = new Box(7); $box->ignored = 'free'; echo $box->value, '|', $box->ignored;",
    ]), "7|free");
}

/// Inline local annotations remain inert, matching AOT's property-only PHPDoc var handling.
#[test]
fn generics_docblocks_leave_inline_local_var_inert() {
    let _guard = crate::strict_php_mode::scoped_enable();
    assert_eq!(run(&[
        "/** @template T */ class Box { /** @param T $value */ public function __construct(public $value) {} public function inspect(): string { /** @var T $local */ $local = 'free'; return $local; } } echo (new Box(7))->inspect();",
    ]), "free");
}

/// Callable aliases, explicit first-class callables, and named callback arrays specialize lazily.
#[test]
fn generics_callbacks_share_specializations() {
    assert_eq!(run(&[
        "function id<T>(T $v): T { return $v; } $f = id<int>(...); echo $f(7), '|'; $f = 'id'; echo $f('alias'), '|', call_user_func('id', 8), '|', call_user_func_array('id', ['v' => 'named']), '|', is_callable('id');",
    ]), "7|alias|8|named|1");
}

/// Generated-code context calls see templates before any interpreter call instantiated them.
#[test]
fn generics_context_entry_points_specialize_templates() {
    let mut context = ElephcEvalContext::new();
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();
    let program = parse_fragment(b"function id<T>(T $v): T { return $v; } class Box<T> { public const LABEL = 'box'; public static int $made = 0; public function __construct(public T $v) {} public static function make(T $v): Box<T> { return new Box<T>($v); } }").unwrap();
    execute_program_with_context(&mut context, &program, &mut scope, &mut values).unwrap();
    let integer = values.int(7).unwrap();
    let result = execute_context_function(&mut context, "id", vec![integer], &mut values).unwrap();
    assert_eq!(values.get(result), FakeValue::Int(7));
    let EvalOutcome::Value(label) = execute_context_class_constant_fetch(&mut context, "Box", "LABEL", &mut values).unwrap() else { panic!("expected constant"); };
    assert_eq!(values.get(label), FakeValue::String("box".into()));
    execute_context_static_property_set(&mut context, "Box", "made", integer, &mut values).unwrap();
    let EvalOutcome::Value(made) = execute_context_static_property_get(&mut context, "Box", "made", &mut values).unwrap() else { panic!("expected static value"); };
    assert_eq!(values.get(made), FakeValue::Int(7));
    let EvalOutcome::Value(object) = execute_context_new_object_outcome(&mut context, "Box", vec![integer], &mut values).unwrap() else { panic!("expected object"); };
    let identity = values.object_identity(object).unwrap();
    assert_eq!(context.dynamic_object_class_name(identity).as_deref(), Some("Box<int>"));
    let string = values.string("ok").unwrap();
    let EvalOutcome::Value(object) = execute_context_static_method_call_outcome(&mut context, "Box", "make", vec![string], &mut values).unwrap() else { panic!("expected object"); };
    let identity = values.object_identity(object).unwrap();
    assert_eq!(context.dynamic_object_class_name(identity).as_deref(), Some("Box<string>"));
    let EvalOutcome::Throwable(error) = execute_context_function_outcome(&mut context, "id<int>", vec![string], &mut values).unwrap() else { panic!("expected type error"); };
    assert!(values.object_is_a(error, "TypeError", false).unwrap());
}

/// Function and class template names occupy separate PHP symbol namespaces.
#[test]
fn generics_function_and_class_namespaces_are_separate() {
    assert_eq!(run(&[
        "function Box<T>(T $v): T { return $v; } class Box<T> { public function __construct(public T $v) {} } function Item(): string { return 'function'; } class Item<T> { public function __construct(public T $v) {} } class Plain {} function Plain<T>(T $v): T { return $v; } echo Box(7), '|', (new Box('class'))->v, '|', Item(), '|', (new Item(8))->v, '|', Plain(9);",
    ]), "7|class|function|8|9");
}

/// Ordinary declarations cannot overwrite a retained template of their symbol kind.
#[test]
fn generics_ordinary_redefinitions_are_rejected() {
    for (declaration, duplicate) in [
        ("function id<T>(T $v): T { return $v; }", "function id($v) { return $v; }"),
        ("class Box<T> {}", "class Box {}"),
        ("interface Read<T> {}", "interface Read {}"),
        ("trait Store<T> {}", "trait Store {}"),
    ] {
        let mut context = ElephcEvalContext::new();
        let mut scope = ElephcEvalScope::new();
        let mut values = FakeOps::default();
        let program = parse_fragment(declaration.as_bytes()).unwrap();
        execute_program_with_context(&mut context, &program, &mut scope, &mut values).unwrap();
        let program = parse_fragment(duplicate.as_bytes()).unwrap();
        assert_eq!(execute_program_with_context(&mut context, &program, &mut scope, &mut values), Err(EvalStatus::RuntimeFatal));
    }
}
