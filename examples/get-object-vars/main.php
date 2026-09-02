<?php
// Inspect the public properties of an object as an associative array.

class Profile {
    public string $name = "Ada";
    public int $score = 42;
    private string $token = "hidden";
}

$properties = get_object_vars(new Profile());

echo $properties["name"], " scored ", $properties["score"], "\n";
