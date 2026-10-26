<?php
class Lease {
    public string $name;
    public function __construct(string $name) { $this->name = $name; }
    public function __destruct() { echo "Released ", $this->name, "\n"; }
}

$leases = ['obj' => new Lease('input'), 'inf' => new Lease('output')];
echo "Work complete\n";
unset($leases);
