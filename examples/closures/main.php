<?php

// Anonymous functions (closures) and arrow functions

// Basic anonymous function assigned to a variable
$double = function($x) { return $x * 2; };
echo "double(5) = ";
echo $double(5);
echo "\n";

// Arrow function (shorthand syntax)
$triple = fn($x) => $x * 3;
echo "triple(4) = ";
echo $triple(4);
echo "\n";

// Multi-parameter closure
$add = function($a, $b) { return $a + $b; };
echo "add(3, 7) = ";
echo $add(3, 7);
echo "\n";

// Arrow function with expression body
$square_plus_one = fn(int $x): int => $x * $x + 1;
echo "square_plus_one(5) = ";
echo $square_plus_one(5);
echo "\n";

// Closures as callbacks to array_map
$values = [1, 2, 3, 4];
$doubled = array_map(fn($x) => $x * 2, $values);
echo "doubled: ";
echo $doubled[0];
echo " ";
echo $doubled[1];
echo " ";
echo $doubled[2];
echo " ";
echo $doubled[3];
echo "\n";

// Closures as callbacks to array_filter
$numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
$evens = array_filter($numbers, fn($n) => $n % 2 == 0);
echo "Even count: ";
echo count($evens);
echo "\n";

// Closures with array_reduce
$sum = array_reduce([1, 2, 3, 4, 5], function($carry, $item) {
    return $carry + $item;
}, 0);
echo "Sum of 1..5 = ";
echo $sum;
echo "\n";

// Closures capturing variables with use()
$factor = 3;
$multiply = function(int $x) use ($factor): int {
    return $x * $factor;
};
echo "multiply(5) with factor=3: ";
echo $multiply(5);
echo "\n";

// Capturing by reference for recursive anonymous functions
$factorial = null;
$factorial = function($n) use (&$factorial) {
    return $n <= 1 ? 1 : $n * $factorial($n - 1);
};
echo "factorial(5) = ";
echo $factorial(5);
echo "\n";

// Captured closures as callback values
$scaled = array_map($multiply, [2, 4, 6]);
echo "scaled values: ";
foreach ($scaled as $value) {
    echo $value;
    echo " ";
}
echo "\n";

// Multiple captures
$prefix = "Result";
$suffix = "!";
$format = function($val) use ($prefix, $suffix) {
    return $prefix . ": " . $val . $suffix;
};
echo $format("42");
echo "\n";

// Closures defined in an instance method auto-bind $this (no use($this) needed).
// The closure sees the live object, so mutations through $this persist.
class Tally {
    private int $total = 0;

    public function adder(): callable {
        return function (int $n): int {
            $this->total += $n;     // mutates the live object
            return $this->total;
        };
    }

    public function reporter(): callable {
        return fn (): string => "total=" . $this->total;  // arrow binds $this too
    }
}

$tally = new Tally();
$add = $tally->adder();
echo "running total: ";
echo $add(5);
echo " ";
echo $add(3);
echo "\n";
echo ($tally->reporter())();
echo "\n";

// A closure's bound $this can be rebound to another object with bindTo /
// Closure::bind, producing a new closure and leaving the original untouched.
class Box {
    public int $value;
    public function __construct(int $value) { $this->value = $value; }
    public function reader(): callable {
        return function (): int { return $this->value; };
    }
}

$first = new Box(10);
$second = new Box(20);
$read = $first->reader();
$rebound = $read->bindTo($second);
echo "rebound: ";
echo $read();      // 10 — original
echo " ";
echo $rebound();   // 20 — rebound to $second
echo "\n";

// call() binds $this and invokes the closure in one step.
echo "call: ";
echo $read->call($second);   // 20 — bound to $second for this call only
echo "\n";

// Closure::bind also works on CAPTURELESS closures and closures with several
// `use (...)` captures — not just a single implicit $this.
$captureless = \Closure::bind(static function (): int { return 7; }, null);
echo "captureless: ";
echo $captureless();
echo "\n";

$make_greeting = function (string $greeting, string $name): string {
    return "$greeting, $name!";
};
$bound_greeting = \Closure::bind($make_greeting, null);
echo "n-capture: ";
echo $bound_greeting("Hello", "World");
echo "\n";

// A literal `$scope` argument to Closure::bind additionally rebinds the closure's
// visibility scope, letting a top-level closure read/write a protected property
// through a parameter typed as (or a subclass of) that scope — the pattern
// Symfony's cache layer uses to update a CacheItem's protected metadata from a
// closure it hands to (and gets called back by) the cache adapter.
class CacheEntry {
    protected int $hits = 0;
    public function reader(): int { return $this->hits; }
}

$record_hit = \Closure::bind(
    static function (CacheEntry $entry): void {
        $entry->hits = $entry->hits + 1;
    },
    null,
    CacheEntry::class
);
$entry = new CacheEntry();
$record_hit($entry);
$record_hit($entry);
echo "scope-rebind hits: ";
echo $entry->reader();
echo "\n";

// Static function-local variables now once-guard their WHOLE initializer, not
// just the final store: a closure default assigned via `static $x; $x ??= ...;`
// is created exactly once — subsequent calls reuse the SAME closure instead of
// silently reconstructing it every time.
function greeter(): callable {
    $prefix = "Hi";
    static $greet;
    $greet ??= function (string $name) use ($prefix): string {
        return "$prefix, $name!";
    };
    return $greet;
}

echo greeter()("Ada"), "\n";
echo greeter()("Grace"), "\n";
