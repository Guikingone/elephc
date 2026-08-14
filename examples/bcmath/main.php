<?php
// Exact invoice arithmetic without binary floating-point rounding.

bcscale(2);

$net = '149.99';
$taxRate = '0.22';
$tax = bcmul($net, $taxRate);
$gross = bcadd($net, $tax);

echo "Net:   ", $net, "\n";
echo "Tax:   ", $tax, "\n";
echo "Gross: ", $gross, "\n";

[$whole, $remainder] = bcdivmod($gross, '10');
echo "Ten-euro units: ", $whole, ", remainder: ", $remainder, "\n";
