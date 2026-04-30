<?php

class Point {
    public int $x;
    public int $y;
    public function __construct(int $x, int $y) {
        $this->x = $x;
        $this->y = $y;
    }
}

$origin = new Point(0, 0);
$origin->x = 3;
$origin->y = 4;

$translated = clone $origin;
$translated->x = $translated->x + 10;

echo "origin    = (";
echo $origin->x;
echo ", ";
echo $origin->y;
echo ")\n";

echo "translated = (";
echo $translated->x;
echo ", ";
echo $translated->y;
echo ")\n";
