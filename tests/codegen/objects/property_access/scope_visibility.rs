//! Purpose:
//! Integration or regression tests for php's SCOPE-dependent answer to one property name on a
//! read: the value-read refusal, the silent `isset()` / `empty()` / `??` probe, and the dynamic
//! property a strict ancestor's private name resolves to outside the class that declared it.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.
//! - Every expectation below was measured against php 8.5.10, including the verbatim `Error`
//!   wording, which carries no scope suffix.

use super::*;

/// Verifies a runtime-name value READ php refuses raises the catchable `Error` instead of
/// reading the slot, while `isset()` answers false without raising.
///
/// `isset($o->{$k})` lowers through the same dynamic read as `$o->{$k}`, so before the fetch mode
/// travelled on the instruction the two could not disagree: refusing the read made `isset()`
/// throw, and accepting it let global scope read a private slot by name. Both readers below prove
/// the slot still holds its default.
#[test]
fn test_runtime_name_read_of_an_inaccessible_property_raises_but_isset_answers_false() {
    let out = compile_and_run(
        r#"<?php
class D { private int $n = 7; public function readN(): int { return $this->n; } }
class Prot { protected int $p = 3; public function readP(): int { return $this->p; } }
class Outsider {
    public function poke(Prot $o, string $key): void {
        var_dump(isset($o->{$key}));
        try { $v = $o->{$key}; echo "no throw:" . $v . ";"; }
        catch (Error $e) { echo $e->getMessage() . ";"; }
    }
}
$d = new D();
$key = "n";
var_dump(isset($d->{$key}));
try { $v = $d->{$key}; echo "no throw:" . $v . ";"; }
catch (Error $e) { echo $e->getMessage() . ";"; }
echo $d->readN() . ";";
$p = new Prot();
(new Outsider())->poke($p, "p");
echo $p->readP();
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nCannot access private property D::$n;7;bool(false)\n\
         Cannot access protected property Prot::$p;3"
    );
}

/// Verifies `empty()` and `??` are silent probes for a property php refuses, exactly like
/// `isset()`.
///
/// php answers `true` and the default without raising. Wiring only `isset()` to the probe fetch
/// mode would have made these two throw where php is silent.
#[test]
fn test_empty_and_coalesce_probe_an_inaccessible_property_without_raising() {
    let out = compile_and_run(
        r#"<?php
class D { private int $n = 7; public function readN(): int { return $this->n; } }
$d = new D();
$key = "n";
var_dump(empty($d->{$key}));
var_dump($d->{$key} ?? "dflt");
echo $d->readN();
"#,
    );
    assert_eq!(out, "bool(true)\nstring(4) \"dflt\"\n7");
}

/// Verifies a strict ancestor's PRIVATE name is a DYNAMIC property from the child scope and from
/// global scope, by runtime name and by plain name alike.
///
/// php 7.4 removed shadow properties: `Base::$n` lives under a mangled key, so `B`'s by-name table
/// does not contain it at all. A read warns `Undefined property: B::$n` and answers null, `isset()`
/// answers false in silence, and the ancestor's own scope still reads its slot. The plain name used
/// to be a compile error here, which is php's answer for a private property declared by the
/// receiver's OWN class, not for an inherited one.
#[test]
fn test_strict_ancestor_private_name_reads_as_a_dynamic_property() {
    let out = compile_and_run_capture(
        r#"<?php
class A { private int $n = 1; public function readA(): int { return $this->n; } }
class B extends A {
    public function childRuntime(string $key): void {
        var_dump(isset($this->{$key}));
        var_dump($this->{$key});
    }
    public function childDirect(): void {
        var_dump(isset($this->n));
        var_dump($this->n);
    }
}
$b = new B();
$b->childRuntime("n");
$b->childDirect();
$key = "n";
var_dump(isset($b->{$key}));
var_dump($b->{$key});
var_dump(isset($b->n));
var_dump($b->n);
echo $b->readA();
"#,
    );
    assert!(out.success, "fixture must not fault: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "bool(false)\nNULL\nbool(false)\nNULL\nbool(false)\nNULL\nbool(false)\nNULL\n1"
    );
    assert_eq!(
        out.stderr.matches("Warning: Undefined property: B::$n").count(),
        4,
        "each VALUE read warns and each probe stays silent: {}",
        out.stderr
    );
}

/// Verifies a dynamic property created under a strict ancestor's private name is what the child
/// and global scopes then read, while the ancestor keeps its own slot.
///
/// The class opts into dynamic properties so the entry has somewhere to live. php prints
/// `int(42)` for every reader below except `A::readA()`, which still answers `1`.
#[test]
fn test_dynamic_entry_under_an_ancestor_private_name_is_read_back() {
    let out = compile_and_run_capture(
        r#"<?php
#[\AllowDynamicProperties]
class A { private int $n = 1; public function readA(): int { return $this->n; } }
class B extends A {}
$b = new B();
$key = "n";
var_dump(isset($b->{$key}));
$b->{$key} = 42;
var_dump(isset($b->{$key}));
var_dump($b->{$key});
var_dump(isset($b->n));
var_dump($b->n);
echo $b->readA();
"#,
    );
    assert!(out.success, "fixture must not fault: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "bool(false)\nbool(true)\nint(42)\nbool(true)\nint(42)\n1"
    );
    assert!(
        !out.stderr.contains("Undefined property"),
        "a present dynamic entry must not warn: {}",
        out.stderr
    );
}

/// Verifies a boxed `Mixed` receiver cannot read private storage by plain name from an unrelated
/// scope, and that its probes stay silent.
///
/// The `Mixed` ladder dispatches on the receiver's runtime class id, so it reached the declared
/// slot with no visibility check at all. That was the last remaining plain-name route into a
/// private slot from global scope.
#[test]
fn test_mixed_receiver_read_of_a_private_property_raises_but_probes_stay_silent() {
    let out = compile_and_run(
        r#"<?php
class Sec { private int $s = 5; public int $open = 1; public function readS(): int { return $this->s; } }
function pick(bool $flag): mixed { return $flag ? new Sec() : 1; }
$m = pick(true);
try { $v = $m->s; echo "no throw:" . $v . ";"; }
catch (Error $e) { echo $e->getMessage() . ";"; }
var_dump(isset($m->s));
var_dump(empty($m->s));
var_dump($m->s ?? "dflt");
var_dump($m->open);
echo (new Sec())->readS();
"#,
    );
    assert_eq!(
        out,
        "Cannot access private property Sec::$s;bool(false)\nbool(true)\n\
         string(4) \"dflt\"\nint(1)\n5"
    );
}

/// Verifies the scope-aware read leaves php's other property answers exactly as they were.
///
/// Two same-named private slots stay apart, a protected property stays reachable down the
/// hierarchy, stdClass and `__get` / `__isset` keep their own dispatch, a get-hooked property
/// still runs its accessor, and a nullsafe read on null still answers null.
#[test]
fn test_scope_aware_reads_preserve_shadowing_hierarchy_magic_and_hooks() {
    let out = compile_and_run(
        r#"<?php
class Base { private int $p = 10; public function readP(): int { return $this->p; } public function readDyn(string $k): int { return $this->{$k}; } }
class Child extends Base { private int $p = 20; public function childP(): int { return $this->p; } public function childDyn(string $k): int { return $this->{$k}; } }
class Par { public int $a = 1; protected int $b = 2; public function readB(): int { return $this->b; } }
class Kid extends Par { public function kidDyn(string $k): int { return $this->{$k}; } }
class Magic {
    private array $bag = ["z" => 9];
    public function __get($n) { return $this->bag[$n] ?? "none"; }
    public function __isset($n) { return isset($this->bag[$n]); }
}
class Hooked { public int $h = 4 { get => $this->h * 2; } }
$c = new Child();
echo $c->readP() . ";" . $c->childP() . ";" . $c->readDyn("p") . ";" . $c->childDyn("p") . ";";
$k = new Kid();
echo $k->a . ";" . $k->readB() . ";" . $k->kidDyn("b") . ";";
$s = new stdClass();
$s->x = 7;
$name = "x";
echo $s->{$name} . ";" . var_export(isset($s->{$name}), true) . ";";
$mg = new Magic();
echo $mg->z . ";" . var_export(isset($mg->z), true) . ";" . var_export(isset($mg->q), true) . ";" . $mg->q . ";";
echo (new Hooked())->h . ";";
$n = null;
var_dump($n?->whatever);
"#,
    );
    assert_eq!(
        out,
        "10;20;10;20;1;2;2;7;true;9;true;false;none;8;NULL\n"
    );
}

/// Verifies `??=` reads its target through the same silent probe, then obeys php's WRITE answer.
///
/// `??=` exists so the target may be absent, so the read half must never raise. php then applies
/// the ordinary write rules: a strict ancestor's private name creates a dynamic property, while a
/// private property declared by the receiver's own class refuses the STORE. Measured against php
/// 8.5.10, which prints exactly the same sequence.
#[test]
fn test_coalesce_assign_probes_the_target_then_obeys_the_write_answer() {
    let out = compile_and_run(
        r#"<?php
class A { private int $m = 1; public function readA(): int { return $this->m; } }
#[\AllowDynamicProperties]
class B extends A {}
class Sec { private int $s = 5; public function readS(): int { return $this->s; } }
function pick(bool $flag): mixed { return $flag ? new Sec() : 1; }
$b = new B();
$key = "m";
$b->{$key} ??= 55;
var_dump($b->{$key}, $b->m, $b->readA());
$b->{$key} ??= 99;
var_dump($b->{$key});
$m = pick(true);
try { $m->s ??= "mx"; var_dump($m->s); }
catch (Error $e) { echo "coalesce assign:" . $e->getMessage() . ";"; }
echo (new Sec())->readS();
"#,
    );
    assert_eq!(
        out,
        "int(55)\nint(55)\nint(1)\nint(55)\ncoalesce assign:Cannot access private property Sec::$s;5"
    );
}

/// Verifies `$o?->{$k}` carries the probe fetch mode through the nullsafe CHAIN lowering.
///
/// A `?->` operand is flattened into a chain before the operand-shaped probe routes ever see it,
/// so `isset()`, `empty()` and `??` reached the chain's ordinary value read and `empty($d?->{$k})`
/// raised where php answers `true`. A null receiver still short-circuits to null without a
/// property-on-null warning, and the plain value read still raises.
#[test]
fn test_nullsafe_runtime_name_probes_stay_silent_through_the_chain() {
    let out = compile_and_run_capture(
        r#"<?php
class D { private int $n = 7; public function readN(): int { return $this->n; } }
function pick(bool $flag): ?D { return $flag ? new D() : null; }
$d = pick(true);
$z = pick(false);
$key = "n";
var_dump(isset($d?->{$key}));
var_dump(empty($d?->{$key}));
var_dump($d?->{$key} ?? "dflt");
var_dump(isset($z?->{$key}));
var_dump(empty($z?->{$key}));
var_dump($z?->{$key} ?? "dflt");
try { var_dump($d?->{$key}); }
catch (Error $e) { echo "read:" . $e->getMessage() . "\n"; }
var_dump($z?->{$key});
echo $d->readN();
"#,
    );
    assert!(out.success, "fixture must not fault: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "bool(false)\nbool(true)\nstring(4) \"dflt\"\nbool(false)\nbool(true)\n\
         string(4) \"dflt\"\nread:Cannot access private property D::$n\nNULL\n7"
    );
    assert!(
        !out.stderr.contains("Attempt to read property"),
        "a nullsafe hop must not warn on its null receiver: {}",
        out.stderr
    );
}

/// Verifies a boxed `Mixed` receiver warns `Undefined property` for a strict ancestor's private
/// name on a value READ, and stays silent for all three probes.
///
/// The `Mixed` runtime-name ladder dispatches on the receiver's class id AND the name, so a
/// scope-dynamic name gets its own arm there. Dropping it instead sent the name to the shared miss
/// arm, which answers `null` without a diagnostic, so php's warning went missing on that one path
/// while the typed-receiver path reported it.
#[test]
fn test_mixed_receiver_runtime_name_ancestor_private_warns_only_on_the_value_read() {
    let out = compile_and_run_capture(
        r#"<?php
class A { private int $n = 1; public function readA(): int { return $this->n; } }
class B extends A {}
function pick(bool $flag): mixed { return $flag ? new B() : 1; }
$m = pick(true);
$key = "n";
var_dump(isset($m->{$key}));
var_dump($m->{$key});
var_dump(empty($m->{$key}));
var_dump($m->{$key} ?? "dflt");
echo (new B())->readA();
"#,
    );
    assert!(out.success, "fixture must not fault: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "bool(false)\nNULL\nbool(true)\nstring(4) \"dflt\"\n1"
    );
    assert_eq!(
        out.stderr.matches("Warning: Undefined property: B::$n").count(),
        1,
        "only the VALUE read warns: {}",
        out.stderr
    );
}

/// Verifies an exception thrown from `__isset` propagates out of `isset()` and `empty()`.
///
/// php's probes suppress its own access and miss diagnostics, not user code: `__isset`, `__get` and
/// property hooks all still run and may throw. This is why `PropertyFetchMode` is a diagnostic
/// selector and NOT an effect narrowing, and why `Op::DynamicPropGet` keeps its conservative
/// `may_throw` / `may_warn` contract in both modes.
#[test]
fn test_exception_from_magic_isset_propagates_out_of_the_probe() {
    let out = compile_and_run(
        r#"<?php
class M {
    public function __isset($name) { throw new Exception("boom " . $name); }
    public function __get($name) { return "g" . $name; }
}
$m = new M();
try { var_dump(isset($m->zz)); } catch (Exception $e) { echo "isset:" . $e->getMessage() . ";"; }
try { var_dump(empty($m->zz)); } catch (Exception $e) { echo "empty:" . $e->getMessage() . ";"; }
echo "done";
"#,
    );
    assert_eq!(out, "isset:boom zz;empty:boom zz;done");
}

/// Verifies a class that declares a magic accessor gets php's answer withheld, never its storage.
///
/// php consults `__get` and `__isset` BEFORE it reports anything: on a class that declares them a
/// private property reached from an unrelated scope answers the accessor, never
/// `Cannot access private property`, and a name php resolves to a dynamic property answers the
/// accessor rather than warning `Undefined property`. This compiler cannot dispatch an accessor
/// for a RUNTIME property name yet, so a read of such a name has three wrong answers available and
/// exactly one safe one:
///   - raising or warning would invent a diagnostic php never reports;
///   - reading the declared slot would hand an unrelated scope the private storage php is hiding;
///   - php `null` is wrong in VALUE only, and exposes nothing.
/// `PropertyNameArm::MagicDeferred` takes the third. The php-correct value (`magic:secret` and
/// `inherited:n`, measured on php 8.5.10) arrives with the dedicated runtime-name magic dispatch
/// phase, which is when this test's expectations change.
///
/// The two declaring scopes below prove the storage is intact and still readable from inside.
#[test]
fn test_magic_accessor_class_withholds_its_answer_without_exposing_the_slot() {
    let out = compile_and_run_capture(
        r#"<?php
class M {
    private int $secret = 5;
    public function __isset($name) { return $name === "secret"; }
    public function __get($name) { return "magic:" . $name; }
    public function readSecret(): int { return $this->secret; }
}
class A { private int $n = 1; public function readA(): int { return $this->n; } }
class B extends A { public function __get($name) { return "inherited:" . $name; } }
$m = new M();
$key = "secret";
try { var_dump($m->{$key}); } catch (Error $e) { echo "ERR:" . $e->getMessage() . ";"; }
var_dump(isset($m->{$key}));
$b = new B();
$name = "n";
var_dump($b->{$name});
echo "inside:" . $m->readSecret() . ":" . $b->readA();
"#,
    );
    assert!(out.success, "fixture must not fault: {}", out.stderr);
    // The private payloads are 5 and 1. Neither may appear in an out-of-scope runtime-name read.
    assert_eq!(
        out.stdout,
        "NULL\nbool(false)\nNULL\ninside:5:1",
        "out-of-scope runtime reads must answer NULL and the declaring methods must still read \
         their own slots"
    );
    assert!(
        !out.stdout.contains("int(5)") && !out.stdout.contains("int(1)"),
        "a private payload must never reach an out-of-scope runtime-name read: {}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("ERR:"),
        "php answers the accessor here, so the access error must not be raised: {}",
        out.stdout
    );
    assert!(
        !out.stderr.contains("Undefined property"),
        "php answers the accessor here, so the undefined-property warning must not be emitted: {}",
        out.stderr
    );
}
