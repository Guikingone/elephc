<?php
$it = new ArrayIterator(["alpha" => 10, "beta" => 20]);
$it["gamma"] = 30;

foreach ($it as $key => $value) {
    echo $key;
    echo "=";
    echo $value;
    echo "\n";
}

$obj = new ArrayObject(["left" => "L", "right" => "R"]);
foreach ($obj as $key => $value) {
    echo $key;
    echo ":";
    echo $value;
    echo "\n";
}

// PHP retains an object backing store; PHP 8.5 deprecates this legacy usage.
$config = new stdClass();
$config->host = "localhost";
$config->port = 8080;

$settings = new ArrayObject($config);
foreach ($settings as $key => $value) {
    echo $key;
    echo "=";
    echo $value;
    echo "\n";
}

// Numeric appends use separate property buckets; a copied array exposes their values.
$settings[] = "legacy setting";
$settingsCopy = $settings->getArrayCopy();
echo "appended=", $settingsCopy[0], "\n";

// Fixed-array textual offsets follow PHP's canonical integer-key spelling.
$fixed = new SplFixedArray(13);
$fixed["12"] = "canonical key";
echo $fixed[12], "\n";
try {
    $fixed["18446744073709551616"] = "oversized";
} catch (TypeError $error) {
    echo "Oversized textual offset rejected\n";
}
