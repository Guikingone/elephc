<?php

// Generic collection code must pass an object offset through unchanged.
final class Labels implements ArrayAccess
{
    private SplObjectStorage $values;

    public function __construct() { $this->values = new SplObjectStorage(); }
    public function offsetExists(mixed $key): bool { return $this->values->offsetExists($key); }
    public function offsetGet(mixed $key): mixed { return $this->values[$key]; }
    public function offsetSet(mixed $key, mixed $value): void { $this->values[$key] = $value; }
    public function offsetUnset(mixed $key): void { $this->values->offsetUnset($key); }
}

function label(mixed $collection, mixed $job): mixed
{
    return $collection[$job];
}

$job = new stdClass();
$labels = new Labels();
$labels[$job] = 'Index build';
echo label($labels, $job), "\n";

// The key is known to be an object here; a generic receiver still gets it unchanged.
function labelForNewJob(mixed $collection): void
{
    $job = new stdClass();
    $collection[$job] = 'Release build';
    echo $collection[$job], "\n";
}

labelForNewJob($labels);
