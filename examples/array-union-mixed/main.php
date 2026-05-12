<?php

// PHP's `+` operator on arrays computes a union with left-key precedence.
// elephc supports it across same-kind operands (indexed+indexed, assoc+assoc)
// and across mixed kinds (indexed+assoc, assoc+indexed). The result of a
// mixed-kind union is always associative-shaped because a string key may have
// entered the result, and numeric-string keys collide with their integer form.

// --- indexed + associative ---
echo "indexed + associative\n";
$indexed_plus_assoc = [1, 2, 3] + ["a" => 10, "b" => 20];
foreach ($indexed_plus_assoc as $k => $v) {
    echo "  $k => $v\n";
}

// --- associative + indexed (left-key precedence keeps 0 => "x") ---
echo "\nassociative + indexed (left-key precedence)\n";
$assoc_plus_indexed = ["a" => 1, 0 => "x"] + [10, 20, 30];
foreach ($assoc_plus_indexed as $k => $v) {
    echo "  $k => $v\n";
}

// --- numeric-string key collision ("0" collapses onto integer 0) ---
echo "\nnumeric-string key collision\n";
$collision = ["0" => "first"] + [10, 20];
foreach ($collision as $k => $v) {
    echo "  $k => $v\n";
}

// --- heterogeneous indexed payload + associative ---
echo "\nheterogeneous indexed payload + associative\n";
$heterogeneous = [1, "two", 3.0, true] + ["x" => "y"];
foreach ($heterogeneous as $k => $v) {
    echo "  $k => $v\n";
}
