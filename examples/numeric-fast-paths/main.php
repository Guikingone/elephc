<?php
// Rolling integer hash benchmark. The EIR optimizer keeps the checked multiply/add
// chain unboxed and in registers, while retaining PHP's overflow-to-float semantics.

$h = 1;
$n = 100000000;

for ($i = 1; $i <= $n; $i++) {
    $h = ($h * 31 + $i) & 0x3fffffff;
}

echo $h, "\n";
