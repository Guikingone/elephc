<?php

// Packed particles integrated in a buffer<T> hot loop. With --ir-opt (the default)
// the loop resolves the buffer handle once before it starts, and every $p[$i]
// access in one iteration shares a single bounds-checked record address.

packed class Particle
{
    public float $x;
    public float $vx;
}

function step(buffer<Particle> $p, int $count, float $dt): void
{
    for ($i = 0; $i < $count; $i++) {
        $x = $p[$i]->x + $p[$i]->vx * $dt;
        if ($x < 0.0 || $x > 100.0) {
            $p[$i]->vx = -$p[$i]->vx;
            $x = $p[$i]->x;
        }
        $p[$i]->x = $x;
    }
}

$count = 8;
$p = buffer_new<Particle>($count);
for ($i = 0; $i < $count; $i++) {
    $p[$i]->x = $i * 12.5;
    $p[$i]->vx = ($i % 3 - 1) * 7.5;
}
for ($frame = 0; $frame < 100; $frame++) {
    step($p, $count, 0.5);
}

$sum = 0.0;
for ($i = 0; $i < $count; $i++) {
    $sum += $p[$i]->x;
}
echo "particles=", $count, "; sum=", $sum, "\n";
buffer_free($p);

// Arrays of buffers: a particle system per layer, stepped and summed through one list. The
// list holds the buffer handles, so foreach and element reads hand the buffers back as they are.
$layers = [];
for ($layer = 0; $layer < 3; $layer++) {
    $particles = buffer_new<Particle>($count);
    for ($i = 0; $i < $count; $i++) {
        $particles[$i]->x = $i * 10.0 + 1.0 * $layer;
        $particles[$i]->vx = 2.5;
    }
    $layers[] = $particles;
}
foreach ($layers as $particles) {
    step($particles, $count, 1.0);
}
$total = 0.0;
foreach ($layers as $layer => $particles) {
    $layerSum = 0.0;
    for ($i = 0; $i < $count; $i++) {
        $layerSum += $particles[$i]->x;
    }
    echo "layer ", $layer, ": sum=", $layerSum, "\n";
    $total += $layerSum;
}
echo "layers=", count($layers), "; total=", $total, "\n";
while (count($layers) > 0) {
    $particles = array_pop($layers);
    buffer_free($particles);
}

// A counting sort over grid cells. `$counts[$cell]++` and `--$ends[...]` update a scalar buffer
// element in place: one typed read, add and store per update, with no allocation in the loop.
$cellCount = 4;
$cellOf = [2, 0, 3, 2, 1, 0, 2, 3];
$items = count($cellOf);
$counts = buffer_new<int>($cellCount);
foreach ($cellOf as $cell) {
    $counts[$cell]++;
}
$ends = buffer_new<int>($cellCount);
$end = 0;
for ($cell = 0; $cell < $cellCount; $cell++) {
    $end += $counts[$cell];
    $ends[$cell] = $end;
}
$order = buffer_new<int>($items);
for ($item = $items - 1; $item >= 0; $item--) {
    $order[--$ends[$cellOf[$item]]] = $item;
}
echo "cell counts:";
for ($cell = 0; $cell < $cellCount; $cell++) {
    echo " ", $counts[$cell];
}
echo "; sorted items:";
for ($i = 0; $i < $items; $i++) {
    echo " ", $order[$i];
}
echo "\n";
buffer_free($counts);
buffer_free($ends);
buffer_free($order);
