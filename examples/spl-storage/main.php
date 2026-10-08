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

// PHP's constructor accepts `array|object`; an object contributes its public properties.
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

