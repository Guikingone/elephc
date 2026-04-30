use super::*;

#[test]
fn test_clone_scalar_properties_are_independent() {
    // Modifying a clone's scalar properties must not affect the source.
    let out = compile_and_run(
        r#"<?php
class Point {
    public int $x;
    public int $y;
    public function __construct(int $x, int $y) { $this->x = $x; $this->y = $y; }
}
$a = new Point(3, 4);
$b = clone $a;
$b->x = 99;
$b->y = 100;
echo $a->x; echo " "; echo $a->y; echo "\n";
echo $b->x; echo " "; echo $b->y; echo "\n";
"#,
    );
    assert_eq!(out, "3 4\n99 100\n");
}

#[test]
fn test_clone_string_property_creates_separate_copy() {
    // Reassigning a clone's string property must not corrupt the source —
    // strings are persisted independently into a fresh heap allocation so
    // freeing the clone's slot cannot dangle the source.
    let out = compile_and_run(
        r#"<?php
class Holder {
    public string $name;
    public function __construct(string $n) { $this->name = $n; }
}
$a = new Holder("alice");
$b = clone $a;
$b->name = "bob";
echo $a->name; echo "\n";
echo $b->name; echo "\n";
"#,
    );
    assert_eq!(out, "alice\nbob\n");
}

#[test]
fn test_clone_returns_distinct_instance() {
    // Clone must produce a new object; identity comparison reports them as
    // different even though every property starts equal.
    let out = compile_and_run(
        r#"<?php
class P { public int $x = 1; }
$a = new P();
$b = clone $a;
if ($a === $b) { echo "same"; } else { echo "different"; }
"#,
    );
    assert_eq!(out, "different");
}

#[test]
fn test_clone_is_shallow_for_subobjects() {
    // PHP's clone is shallow: nested objects are shared by reference between
    // source and clone, so mutating the inner object is visible from both.
    let out = compile_and_run(
        r#"<?php
class Inner {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
}
class Outer {
    public Inner $inner;
    public function __construct(Inner $i) { $this->inner = $i; }
}
$a = new Outer(new Inner(10));
$b = clone $a;
$b->inner->v = 99;
echo $a->inner->v; echo " "; echo $b->inner->v; echo "\n";
"#,
    );
    assert_eq!(out, "99 99\n");
}

#[test]
fn test_clone_array_property_uses_copy_on_write() {
    // Array properties are refcount-shared after clone; appending an element
    // to the clone's array triggers copy-on-write so the source stays intact.
    let out = compile_and_run(
        r#"<?php
class Box {
    public array $items;
    public function __construct() { $this->items = []; }
}
$a = new Box();
$a->items[] = 10;
$a->items[] = 20;
$b = clone $a;
$b->items[] = 99;
echo count($a->items); echo " "; echo count($b->items); echo "\n";
"#,
    );
    assert_eq!(out, "2 3\n");
}

#[test]
fn test_clone_does_not_invoke_constructor() {
    // PHP semantics: __construct must not run on the cloned object.
    // We prove this by making the constructor write to a global counter.
    let out = compile_and_run(
        r#"<?php
class Tracker {
    public int $built;
    public function __construct() { $this->built = 1; }
}
$a = new Tracker();
$a->built = 7;
$b = clone $a;
echo $b->built;
"#,
    );
    assert_eq!(out, "7");
}
