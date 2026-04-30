---
title: "Classes"
description: "Classes, interfaces, abstract classes, traits, enums, properties, and inheritance."
sidebar:
  order: 8
---

## Class declaration
```php
<?php
class Point {
    public $x;
    public $y;

    public function __construct($x, $y) {
        $this->x = $x;
        $this->y = $y;
    }

    public function magnitude() {
        return sqrt($this->x * $this->x + $this->y * $this->y);
    }

    public static function origin() {
        return new Point(0, 0);
    }
}
```

## Interfaces
```php
<?php
interface Named {
    public function name();
}

class Product implements Named {
    public function name() { return "widget"; }
    public function label() { return strtoupper($this->name()); }
}
```
- signature-only methods, no bodies, no properties
- interface inheritance flattened transitively with cycle detection

## Type checks with instanceof
```php
<?php
interface Renderable {
    public function render();
}

class Widget {
    public function render() { return "widget"; }
}

class Button extends Widget implements Renderable {}

$item = new Button();
echo ($item instanceof Button) ? "yes" : "no";      // yes
echo ($item instanceof Widget) ? "yes" : "no";      // yes
echo ($item instanceof Renderable) ? "yes" : "no";  // yes
```

The runtime check uses emitted class metadata, so subclasses match parent classes and implemented interfaces. The left-hand side may be a direct object or a boxed `mixed` / nullable / union value; non-object payloads return `false`. Supported targets are named classes/interfaces, `self`, `parent`, and late-bound `static`.

## Abstract classes
```php
<?php
abstract class BaseGreeter {
    abstract public function label();
    public function greet() { return "hi " . $this->label(); }
}
```
- cannot be instantiated
- abstract methods must be bodyless
- non-abstract classes may not have abstract methods

## Final classes, methods, and properties
```php
<?php
final class InvoiceNumber {
    final public $value = 42;

    final public function label() {
        return "invoice:" . $this->value;
    }
}
```
- `final class` cannot be extended
- `final` methods cannot be overridden by subclasses
- `final` properties cannot be redeclared by subclasses
- `final` does not change object layout or dispatch for normal calls
- `abstract final` classes and methods are rejected
- `final private` methods emit a warning, matching PHP, because private methods are not overridden; `__construct` is the exception
- `final private` properties are rejected, matching PHP

## Properties
- `public`, `protected`, `private` visibility
- Optional default values
- Optional type declarations, for example `public int $id` or `public ?string $email = null`
- `readonly` properties (only assigned in `__construct`)
- `final` properties, which can be read normally but cannot be redeclared by subclasses
- Static properties with `public static`, `protected static`, or `private static`, including typed static properties
- `readonly class` makes all properties readonly

```php
<?php
class User {
    public int $id;
    public string $name = "Ada";
    public ?string $email = null;

    public function __construct($id) {
        $this->id = $id;
    }
}
```

Property type declarations are checked at compile time for both instance and static properties. Defaults and later assignments must be compatible with the declared type, including constructor assignments through untyped parameters. Nullable shorthand (`?T`) and union storage use the compiler's boxed mixed representation internally. `void` and `callable` property types are rejected.

## Static properties
Static properties use class-scoped storage and are accessed with `::`.

```php
<?php
class Counter {
    public static int $count = 1;

    public static function bump() {
        self::$count = self::$count + 1;
        return self::$count;
    }
}

echo Counter::$count; // 1
Counter::$count = 5;
echo Counter::bump(); // 6
```

Supported receivers are `ClassName::$prop`, `self::$prop`, `parent::$prop`, and `static::$prop`. Static property visibility and declared types are checked at compile time. Inherited static properties share the declaring class storage until a subclass redeclares the property. Redeclarations follow PHP rules: non-private inherited properties keep invariant declared types, cannot reduce visibility, and cannot override `final` properties. Private static properties redeclared in subclasses are independent slots; `static::$prop` is still late-bound and reports a fatal runtime error if the current method scope cannot access the matched private slot.

Static array properties support direct element writes:

```php
<?php
class Registry {
    public static array $items = [];
}

Registry::$items[] = 10;
Registry::$items[0] = 12;
echo Registry::$items[0]; // 12
```

## Constructor
Called automatically with `new`:
```php
$p = new Point(3, 4);
```

Constructor property promotion is supported. Visibility or `readonly` before a constructor parameter declares a property and assigns the incoming argument at the start of `__construct`.

```php
<?php
class User {
    public function __construct(
        public int $id,
        private string $name = "Ada",
        readonly ?int $rank = null
    ) {}

    public function name() {
        return $this->name;
    }
}

$user = new User(7);
echo $user->id;      // 7
echo $user->name();  // Ada
```

Promoted properties support `public`, `protected`, `private`, `readonly`, nullable and union type declarations, constructor parameter defaults, and by-reference parameters. Variadic promotion is rejected, matching PHP.

By-reference promoted properties are supported when the constructor argument is a variable:

```php
<?php
class Counter {
    public function __construct(public int &$value) {}
}

$value = 1;
$counter = new Counter($value);

$value = 2;
echo $counter->value;  // 2

$counter->value = 3;
echo $value;           // 3
```

Current limitations for by-reference promotion: the promoted property cannot be `readonly`, and by-reference promoted parameters cannot use default values yet.

## Instance methods and $this
Virtual dispatch for overrides.
Private methods are not virtual.

## Nullsafe access
Use `?->` when a receiver may be `null`:

```php
<?php
echo $user?->profile?->name ?? "anonymous";
echo $user?->profile?->label() ?? "missing";
```

When the receiver is `null`, elephc skips the property read or method call and returns `null`. Method arguments on the skipped branch are not evaluated.

## parent::method()
Direct parent implementation call.

## self::method()
Binds to lexical class, not runtime child.

## static::method()
Late static binding — resolves against called class at runtime.

## Static methods
Called with `::`, no `$this`.

## Class name reflection (`::class`)

`::class` returns the fully-qualified class name as a string at compile time.

```php
<?php
namespace App;
class Logger {
    public static function tag() {
        return self::class;          // "App\Logger"
    }
}
echo Logger::class;                  // "App\Logger"
echo \App\Logger::class;             // "App\Logger"
```

Supported receivers: `Class::class`, `\Vendor\Class::class`, `self::class`, `parent::class`, `static::class`.

`static::class` follows PHP late static binding and resolves to the called class.

## Late static binding constructors (`new self()`, `new static()`, `new parent()`)

The `new self()`, `new static()`, and `new parent()` factory patterns are supported inside class methods:

```php
<?php
class Box {
    public string $label = "default";
    public static function make(): Box {
        return new self();
    }
}
$b = Box::make();
echo $b->label;                      // "default"

class Base {
    public string $kind = "base";
}

class Child extends Base {
    public static function makeBase(): Base {
        return new parent();
    }
}
```

`new static()` follows PHP late static binding and constructs an instance of the called class.

## Override rules
Same parameter count, same pass-by-reference positions, same default layout, same variadic shape.

## Traits
Flattened at compile time. Support: `use Trait;`, multiple traits, `insteadof`, `as`, trait properties, static trait methods.

## Property access
`->` for properties and methods.

## Enums
```php
<?php
enum Color: int {
    case Red = 1;
    case Green = 2;
}
echo Color::Red->value;          // 1
echo Color::from(2) === Color::Green; // 1
```
Pure and backed enums. `->value`, `::from()`, `::tryFrom()`, `::cases()`. Only `int` and `string` backing types.

## Magic methods
- `__toString()` — string coercion
- `__get($name)` — reading undefined property
- `__set($name, $value)` — writing undefined property

## Cloning objects
The `clone` keyword performs a shallow copy of an object: a new instance is allocated, every property slot is copied from the source, and refcounted payloads (arrays, objects, mixed) are retained so that source and clone share child storage until one of them mutates it.

```php
<?php
class Point {
    public int $x;
    public int $y;
    public function __construct(int $x, int $y) { $this->x = $x; $this->y = $y; }
}

$a = new Point(3, 4);
$b = clone $a;
$b->x = 99;
echo $a->x;   // 3
echo $b->x;   // 99
```

Semantics:
- `__construct` is **not** invoked on the clone — properties are copied verbatim.
- Scalar properties (`int`, `float`, `bool`) are independent after clone.
- `string` properties are persisted into a fresh heap allocation per clone, so reassigning the clone's slot never dangles the source.
- `array` properties share storage with copy-on-write, matching PHP semantics.
- Object properties are aliased: mutating a nested object is visible from both the source and the clone (PHP's shallow-clone behavior).

The operand of `clone` must evaluate to an object value; `clone 42`, `clone "x"`, and `clone null` are rejected at compile time.

If the class — or any of its ancestors — declares `__clone()`, elephc invokes it on the new instance after the property copy completes. The hook takes no arguments, returns void implicitly, and may use any visibility:

```php
<?php
class Counter {
    public int $value;
    public function __construct(int $value) { $this->value = $value; }
    public function __clone(): void {
        $this->value = $this->value * 10;
    }
}

$a = new Counter(5);
$b = clone $a;
echo $a->value; // 5
echo $b->value; // 50
```

`__clone` runs only on the freshly cloned object, never on the source. Inherited implementations are dispatched: `clone $child` calls the closest `__clone` in the inheritance chain. The `__construct` body is still skipped — only `__clone` is fired by `clone`.

## Limitations
- No abstract properties
- No `readonly static` properties
- No `readonly` or default-valued by-reference promoted properties
- No instance property redeclaration across inheritance chain
