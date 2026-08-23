<?php

// Local retype in permissive mode (the default).
//
// elephc keeps every local monomorphic, but an undeclared-type local may
// change type in three ways. The implicit shapes (2 and 3 below) warn by
// default and become hard errors under --strict-locals; the explicit
// unset() kill (shape 1) is PHP-truthful in both modes. Declared types
// (int $x = ..., typed parameters, properties) stay strict everywhere,
// and reading a variable after a straight-line unset() is a compile
// error — probe with isset() instead.

// -- 1. Explicit kill: unset() ends the binding, the next write starts fresh --
// A scratch buffer holds a status string, is disposed of, and the same name
// is then reused for a counter. No warning: the rebind is a fresh binding.
$scratch = "step " . $argc;
echo strtoupper($scratch), "\n";
unset($scratch);
$scratch = $argc * 10;
echo $scratch + 5, "\n";

// -- 2. Straight-line retype: parse-then-normalize through one name --
// $raw arrives as text (think CLI input) and is normalized in place to the
// number it contains. The right-hand side still reads the old string
// binding; the store re-binds $raw to a fresh int slot. Warns:
//   $raw changes type from string to int; the previous value is discarded
$raw = "42" . $argc;
$raw = (int)$raw;
echo $raw + 1, "\n";

// -- 3. Branch-divergent assignment: one name, two types, boxed storage --
// A label is numeric when arguments were passed and a placeholder string
// otherwise. The whole frame slot becomes boxed mixed storage, so every
// read of $label dispatches through the box. Warns:
//   $label is assigned incompatible types (int and string); it is compiled
//   as boxed mixed storage
if ($argc > 1) {
    $label = (int)$argc;
} else {
    $label = "none";
}
echo "label: ", $label, "\n";
