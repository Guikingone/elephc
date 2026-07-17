<?php

namespace App\Demo;

// __DIR__ in include paths — the idiomatic PHP way to load a sibling file
// regardless of where the script was launched from.
require __DIR__ . '/lib/helper.php';

// File-related magic constants — useful for finding sibling files relative to
// the currently-executing source file.
echo "__FILE__ = " . __FILE__ . "\n";
echo "__DIR__  = " . __DIR__ . "\n";
echo "__LINE__ = " . __LINE__ . "\n";
echo "__NAMESPACE__ = " . __NAMESPACE__ . "\n";

// Inside a free function, __FUNCTION__ is namespace-qualified.
function greet() {
    echo "  in greet():  __FUNCTION__ = " . __FUNCTION__ . "\n";
    echo "  in greet():  __METHOD__   = " . __METHOD__ . "\n";
}
greet();

// Inside a method, __CLASS__ is the FQN class, __METHOD__ is "Class::method".
class Greeter {
    public function hello() {
        echo "  in Greeter::hello():\n";
        echo "    __CLASS__    = " . __CLASS__ . "\n";
        echo "    __METHOD__   = " . __METHOD__ . "\n";
        echo "    __FUNCTION__ = " . __FUNCTION__ . "\n";
    }
}
$g = new Greeter();
$g->hello();

// Magic constants also work in class-constant initializers and property
// defaults: __CLASS__ is the declaring class FQN, and __FUNCTION__/__METHOD__
// are empty because there is no enclosing function. This is exactly the shape
// symfony/var-dumper uses (const X = ['Closure' => __CLASS__.'::method']).
class Registry {
    const OWNER = __CLASS__;
    const INFO  = ['self' => __CLASS__ . '::describe'];
    public string $label = __CLASS__;
}
echo "  Registry::OWNER      = " . Registry::OWNER . "\n";
echo "  Registry::INFO[self] = " . Registry::INFO['self'] . "\n";
echo "  (new Registry)->label = " . (new Registry())->label . "\n";

// Inside a trait method, __CLASS__ is rebound to the class that uses the
// trait, while __METHOD__ and __TRAIT__ keep the trait declaration identity.
trait Reportable {
    public function report() {
        echo "  in Reportable::report():\n";
        echo "    __CLASS__    = " . __CLASS__ . "\n";
        echo "    __METHOD__   = " . __METHOD__ . "\n";
        echo "    __TRAIT__    = " . __TRAIT__ . "\n";
    }
}
class Service {
    use Reportable;
}
$s = new Service();
$s->report();

// Inside a closure, __FUNCTION__ uses PHP's closure marker with file + line.
$f = function() {
    echo "  inside closure: __FUNCTION__ = " . __FUNCTION__ . "\n";
};
$f();

// Magic constant names are case-insensitive, like PHP.
echo "lowercase __dir__ = " . __dir__ . "\n";
