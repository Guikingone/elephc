<?php
// Control-flow normalization v2: every loop, switch, and branch below is written in a
// shape the optimizer canonicalizes before the CFG-aware passes run. The program prints the
// same thing with or without the rewrites; compile with `--emit-ir` to see the simpler shells.

// A `for` without an update clause is a `while` loop; its leading break guard becomes the
// loop test, and the guard's `else` block leads the remaining body.
$queue = [3, 1, 4, 1, 5, 9, 2, 6];
$taken = [];
for ($i = 0; $i < count($queue);) {
    if ($queue[$i] > 5) {
        break;
    } else {
        $taken[] = $queue[$i];
    }
    $i++;
}
echo "Taken before the first value above 5: " . implode(", ", $taken) . "\n";

// An endless loop that ends in a break guard is rotated into `do { ... } while (test)`:
// the body always runs once and the exit is tested after it.
$n = 27;
$steps = 0;
while (true) {
    $n = $n % 2 == 0 ? intdiv($n, 2) : 3 * $n + 1;
    $steps++;
    if ($n == 1) {
        break;
    }
}
echo "Collatz(27) reaches 1 after " . $steps . " steps\n";

// A trailing `continue` reaches the update clause exactly as falling off the body does, so
// it disappears; the update still runs for that iteration.
$evens = 0;
for ($k = 1; $k <= 10; $k++) {
    if ($k % 2 != 0) {
        continue;
    }
    $evens++;
    if ($k == 10) {
        echo "Counted " . $evens . " even numbers\n";
        continue;
    }
}

// `do ... while (true)` is the same loop as `while (true)`.
$attempt = 0;
do {
    $attempt++;
    if ($attempt >= 3) {
        break;
    }
    echo "Attempt " . $attempt . " failed, retrying\n";
} while (true);
echo "Succeeded on attempt " . $attempt . "\n";

// The body that runs last in a `switch` leaves it by falling off the end, so its trailing
// `break` is dropped while every earlier case keeps its own.
function describe(int $code): string {
    switch ($code) {
        case 200:
            return "ok";
        case 404:
            return "missing";
        default:
            $label = "other";
            break;
    }
    return $label;
}
echo describe(200) . ", " . describe(404) . ", " . describe(500) . "\n";

// `if (!c) { A } else { B }` is tested the positive way round: `if (c) { B } else { A }`.
function classify(int $temperature): string {
    if (!($temperature > 25)) {
        return "cool";
    } else {
        return "warm";
    }
}
echo "18C is " . classify(18) . ", 30C is " . classify(30) . "\n";

// A bare `return;` that ends a function body returns null exactly as falling off does, and is
// dropped from every trailing branch; the `finally` still runs on the way out.
function report(int $count): void {
    try {
        if ($count > 1) {
            echo $count . " items";
            return;
        }
        echo "1 item";
        return;
    } finally {
        echo "\n";
    }
}
report(1);
report(3);
