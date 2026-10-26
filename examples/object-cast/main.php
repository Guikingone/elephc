<?php
// The (object) cast: turning configuration arrays into property bags.
//
// A common shape in PHP libraries is to accept an options array and hand it on as an object,
// so callers read `$options->timeout` instead of `$options['timeout']`.

function settings(string $host, int $port, bool $secure): stdClass
{
    return (object) ["host" => $host, "port" => $port, "secure" => $secure];
}

$config = settings("localhost", 6432, true);

echo "host:   " . $config->host . "\n";
echo "port:   " . $config->port . "\n";
echo "secure: " . ($config->secure ? "yes" : "no") . "\n";

// Array keys become property names, so the object round-trips back to the same array.
$back = (array) $config;
echo "keys:   " . implode(", ", array_keys($back)) . "\n";

// A non-array value lands on PHP's `scalar` property.
$wrapped = (object) "just a string";
echo "scalar: " . $wrapped->scalar . "\n";

// null becomes an empty object rather than one holding a null `scalar`.
$empty = (object) null;
echo "empty:  " . count(get_object_vars($empty)) . " properties\n";

// An object is returned unchanged — the cast is the identity, not a copy.
class Endpoint
{
    public function __construct(public string $url) {}
}

$endpoint = new Endpoint("https://example.test");
$same = (object) $endpoint;
$same->url = "https://changed.test";

echo "same:   " . ($same === $endpoint ? "yes" : "no") . "\n";
echo "url:    " . $endpoint->url . "\n";
