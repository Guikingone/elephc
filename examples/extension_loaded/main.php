<?php
// extension_loaded(): does a named PHP extension count as loaded?
// In Elephc's AOT model this resolves against a compile-time-known set and
// const-folds to a bool. Matching is case-insensitive on the canonical names.

echo extension_loaded("json") ? "json: yes\n" : "json: no\n";
echo extension_loaded("JSON") ? "JSON: yes\n" : "JSON: no\n";
echo extension_loaded("Zend OPcache") ? "opcache(canonical): yes\n" : "opcache(canonical): no\n";
echo extension_loaded("zend opcache") ? "opcache(lower): yes\n" : "opcache(lower): no\n";
echo extension_loaded("opcache") ? "opcache(alias): yes\n" : "opcache(alias): no\n";
echo extension_loaded("Reflection") ? "Reflection: yes\n" : "Reflection: no\n";
echo extension_loaded("curl") ? "curl: yes\n" : "curl: no\n";
echo extension_loaded("gd") ? "gd: yes\n" : "gd: no\n";

echo function_exists("extension_loaded") ? "declared: yes\n" : "declared: no\n";

// get_loaded_extensions(): the full list backing extension_loaded().
$ext = get_loaded_extensions();
echo "loaded count: " . count($ext) . "\n";
echo "first: " . $ext[0] . "\n";
echo "has json: " . (in_array("json", $ext) ? "yes" : "no") . "\n";
echo "has Zend OPcache: " . (in_array("Zend OPcache", $ext) ? "yes" : "no") . "\n";
echo "has curl: " . (in_array("curl", $ext) ? "yes" : "no") . "\n";

// get_loaded_extensions(true): just the Zend extensions.
$zend = get_loaded_extensions(true);
echo "zend count: " . count($zend) . "\n";
echo "zend first: " . $zend[0] . "\n";
