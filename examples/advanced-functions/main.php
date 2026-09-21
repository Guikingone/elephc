<?php
// Advanced functions: global variables, static variables, pass by reference

// --- Global variables ---
// Share state between main scope and functions

$total = 0;

function add_to_total($amount) {
    global $total;
    $total = $total + $amount;
}

add_to_total(10);
add_to_total(25);
add_to_total(15);
echo "Total: " . $total . "\n";

// --- Static variables ---
// Persistent state across function calls

function make_id() {
    static $next_id = 0;
    $next_id++;
    return $next_id;
}

echo "ID: " . make_id() . "\n";
echo "ID: " . make_id() . "\n";
echo "ID: " . make_id() . "\n";

// A static declared without an initializer starts as null, like `static $x = null;`
function uninitialized_static() {
    static $seen;
    if ($seen === null) {
        echo "static without initializer starts as null\n";
    }
}

uninitialized_static();

// --- Pass by reference ---
// Modify the caller's variables directly

function clamp_ref(&$val, $lo, $hi) {
    if ($val < $lo) {
        $val = $lo;
    }
    if ($val > $hi) {
        $val = $hi;
    }
}

$temperature = 150;
clamp_ref($temperature, 0, 100);
echo "Clamped: " . $temperature . "\n";

$raw_score = 5;
$score_alias =& $raw_score;
$score_alias = 7;
echo "Aliased score: " . $raw_score . "\n";

// --- Combining features ---
// A simple accumulator using global + reference + static

$log_count = 0;

function log_message(&$count, $msg) {
    static $prefix = 0;
    $prefix++;
    $count++;
    echo "[" . $prefix . "] " . $msg . "\n";
}

log_message($log_count, "Starting up");
log_message($log_count, "Processing");
log_message($log_count, "Done");
echo "Logged " . $log_count . " messages\n";

// --- Argument introspection ---
// PHP lets a function be called with more positional arguments than it declares.
// The surplus is reachable only through func_num_args(), func_get_args() and
// func_get_arg().

function describe_call() {
    $parts = [];
    foreach (func_get_args() as $arg) {
        $parts[] = var_export($arg, true);
    }
    return func_num_args() . " arg(s): " . implode(", ", $parts);
}

echo describe_call() . "\n";
echo describe_call(1, "two", 3.0) . "\n";

// Declared parameters are part of the argument list too, and func_get_arg()
// reads any position by index.
function tag($label) {
    $extra = func_num_args() > 1 ? func_get_arg(1) : "(none)";
    return $label . " -> " . $extra;
}

echo tag("first") . "\n";
echo tag("first", "second") . "\n";

// --- An untyped parameter reached only through a callable ---
// A parameter with no type hint takes whatever the caller passes. When every call goes through
// a callable rather than naming the function, there is no call site to infer that type from, so
// the return stays open instead of being pinned to whatever the compiler guessed first --
// a returned string comes back as a string, not as int(0).

function passthrough($value) { return $value; }

$passthrough_fcc = passthrough(...);
echo "callable-only passthrough (string): " . $passthrough_fcc("kept as a string") . "\n";
echo "callable-only passthrough (int): " . $passthrough_fcc(42) . "\n";
$passthrough_name = "passthrough";
echo "callable-only passthrough (by name): " . call_user_func($passthrough_name, "also kept") . "\n";
echo "callable-only passthrough (mapped): " . implode(", ", array_map($passthrough_fcc, ["a", "b"])) . "\n";
