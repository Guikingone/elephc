<?php

// PHP's internal array pointer: key(), current(), next(), prev(), reset(), end().
//
// Every array carries a cursor. reset()/end() park it on the first/last element,
// next()/prev() step it, and key()/current() read the key and value under it.
// Once the cursor runs off either end it stays invalid until reset() or end()
// brings it back -- stepping the other way does NOT walk back in.

$readings = [
    "monday"    => 18.5,
    "tuesday"   => 21.0,
    "wednesday" => 19.75,
    "thursday"  => 23.5,
    "friday"    => 22.25,
];

// The classic cursor walk. An array starts with its pointer on the first
// element, so the reset() here is documentation rather than necessity.
echo "Week in order:\n";
reset($readings);
while (($celsius = current($readings)) !== false) {
    printf("  %-10s %5.2f C\n", key($readings), $celsius);
    next($readings);
}

// end() parks on the last element; prev() walks backwards from there.
echo "\nLast three, backwards:\n";
end($readings);
for ($seen = 0; $seen < 3; $seen++) {
    printf("  %-10s %5.2f C\n", key($readings), current($readings));
    prev($readings);
}

// Walking off the front leaves the cursor invalid: current() is false and
// key() is null. next() does not recover it -- only reset()/end() do.
echo "\nOff the front:\n";
reset($readings);
prev($readings);
var_dump(current($readings));
var_dump(key($readings));
var_dump(next($readings));

echo "\nBack on the rails: ";
var_dump(reset($readings));

// foreach never disturbs the pointer, because PHP iterates an internal copy.
next($readings);
$before = key($readings);
$total = 0.0;
foreach ($readings as $celsius) {
    $total += $celsius;
}
printf("\nMean %.2f C, and the cursor is still on %s (was %s).\n",
    $total / count($readings), key($readings), $before);
