<?php

$url = "https://alice:secret@example.com:8443/products?id=42#details";
$parts = parse_url($url);

echo "Host: ", parse_url($url, PHP_URL_HOST), "\n";
echo "Port: ", parse_url($url, PHP_URL_PORT), "\n";
echo "Path: ", $parts["path"], "\n";
echo "Query: ", $parts["query"], "\n";

$relative = parse_url("//cdn.example.com/assets/app.js");
echo "Scheme-relative host: ", $relative["host"], "\n";
