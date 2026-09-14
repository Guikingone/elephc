<?php

// Local retype in permissive mode (the default).
//
// elephc keeps every local monomorphic, but a local may change type in
// three ways. The implicit shapes (2 and 3 below) warn by default and
// become hard errors under --strict-locals; the explicit unset() kill
// (shape 1) is not gated by the flag and behaves identically in both
// modes. Its unbinding follows PHP's own model, but the consequence
// diverges: PHP warns on a later read and evaluates it as null, where
// elephc rejects that read at compile time — probe with isset() instead.
// Declared types (int $x = ..., properties) stay strict everywhere; a
// PARAMETER TYPE HINT does not, because it constrains the incoming
// argument rather than the local (shape 4 below).

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
//   $raw changes type from string to int; the previous value is
//   discarded (compile with --strict-locals to make this an error)
$raw = "42" . $argc;
$raw = (int)$raw;
echo $raw + 1, "\n";

// -- 3. Branch-divergent assignment: one name, two types, boxed storage --
// A label is numeric when arguments were passed and a placeholder string
// otherwise. The whole frame slot becomes boxed mixed storage, so every
// read of $label dispatches through the box. Warns:
//   $label is assigned incompatible types (int and string); it is compiled
//   as boxed mixed storage (compile with --strict-locals to make this an
//   error)
if ($argc > 1) {
    $label = (int)$argc;
} else {
    $label = "none";
}
echo "label: ", $label, "\n";

// -- 4. A typed parameter is a local too --
// PHP's `float $value` constrains what callers may pass; it says nothing
// about the variable afterwards, so formatting in place through the same
// name is valid PHP. Warns like any other straight-line retype:
//   $value changes type from float to string; the previous value is
//   discarded (compile with --strict-locals to make this an error)
// The hint still governs the call boundary — `format("x")` is still
// rejected — and a by-REFERENCE parameter is still never retypable,
// because the caller's storage is reachable through it.
function format(float $value): string {
    $value = number_format($value, 2);
    return $value;
}
echo format($argc + 0.5), "\n";
