<?php
// The (object) cast — converting values to stdClass.

// An associative array becomes a stdClass: keys turn into property names.
$config = (object)["host" => "localhost", "port" => 8080];
echo $config->host, ":", $config->port, "\n";   // localhost:8080

// An indexed array uses the integer keys (as strings) for property names.
$pair = (object)["a", "b"];
echo $pair->{"0"}, $pair->{"1"}, "\n";           // ab

// A scalar is wrapped in a single `scalar` property.
$wrapped = (object)42;
echo $wrapped->scalar, "\n";                      // 42

// null becomes an empty stdClass, just like `new stdClass()`.
$empty = (object)null;
$empty->ready = true;
echo $empty->ready ? "ready\n" : "no\n";          // ready

// Casting an existing object returns the SAME instance (no copy):
// mutating the result is visible through the original variable.
$original = new stdClass();
$original->count = 1;
$alias = (object)$original;
$alias->count = 2;
echo $original->count, "\n";                       // 2
