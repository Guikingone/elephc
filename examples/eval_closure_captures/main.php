<?php
// Reflection keeps a closure and its captured resource owner alive until it is released.
$source = <<<'PHP'
class ResourceOwner {
    public function __destruct() { echo "released\n"; }
}
$owner = new ResourceOwner();
$callback = function () use ($owner) { echo "working\n"; };
$reflection = new ReflectionFunction($callback);
unset($owner, $callback);
$retained = $reflection->getClosure();
$copy = clone $retained;
unset($reflection, $retained);
$copy();
unset($copy);
PHP;
$source .= " // " . $argc;
eval($source);
unset($source);
