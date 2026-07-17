<?php

// Natural-order comparison keeps embedded numbers in human order:
// "img2" sorts before "img10" even though '1' < '2' lexically.
$pairs = [["img2.png", "img10.png"], ["v1.9", "v1.10"], ["file001", "file10"]];
foreach ($pairs as [$a, $b]) {
    $cmp = strnatcmp($a, $b);
    $rel = $cmp < 0 ? "<" : ($cmp > 0 ? ">" : "==");
    echo "$a $rel $b\n";
}

// strnatcasecmp ignores ASCII case while keeping natural numeric order.
echo "IMG10 vs img2 (ci): ", strnatcasecmp("IMG10", "img2"), "\n";

// array_is_list distinguishes dense 0..n-1 lists from associative maps.
$list = ["a", "b", "c"];
$map = ["host" => "localhost", "port" => 5432];
echo "list is list? ", array_is_list($list) ? "yes" : "no", "\n";
echo "map is list? ", array_is_list($map) ? "yes" : "no", "\n";

// array_replace merges configuration overrides last-wins, preserving keys.
$defaults = ["host" => "localhost", "port" => 5432, "ssl" => "off"];
$overrides = ["port" => 6379, "ssl" => "on"];
$config = array_replace($defaults, $overrides);
foreach ($config as $key => $value) {
    echo "  $key = $value\n";
}
