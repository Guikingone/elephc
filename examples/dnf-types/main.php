<?php

// PHP 8.2 DNF (disjunctive normal form) types: a parenthesized intersection group
// used as a member of a union. Here `(HasSize&Renderable)|null` means "either a value
// that is BOTH HasSize and Renderable, or null".

interface HasSize {}
interface Renderable {}

// A concrete type that satisfies both interfaces, so it fits the intersection arm.
class Bag implements HasSize, Renderable {}

class Holder {
    // DNF property type: null by default, later filled with a value satisfying (HasSize&Renderable).
    protected (HasSize&Renderable)|null $item = null;

    public function fill(Bag $b): void {
        $this->item = $b;
    }

    public function isFilled(): bool {
        return $this->item !== null;
    }
}

$h = new Holder();
echo $h->isFilled() ? "y" : "n";   // n — starts null
$h->fill(new Bag());
echo $h->isFilled() ? "y" : "n";   // y — now holds a (HasSize&Renderable)
echo "\n";
