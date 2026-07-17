<?php

// Demonstrates elephc's C-style string escaping and coercion builtins.

// strval() coerces any scalar to its PHP string representation.
$parts = [strval(42), strval(3.14), strval(true), strval(null)];
echo "strval: [", implode("][", $parts), "]\n";

// strrchr() returns the tail of a string from the last occurrence of a byte —
// handy for pulling a file extension off a path.
$path = "/var/www/app/index.php";
echo "extension: ", strrchr($path, "."), "\n";
echo "basename:  ", substr(strrchr($path, "/"), 1), "\n";

// addcslashes() escapes a chosen character set (ranges like "A..Z" are allowed),
// turning control bytes into readable C escapes.
$raw = "Line1\tCol\nLine2";
$escaped = addcslashes($raw, "\0..\37");
echo "escaped: ", $escaped, "\n";

// stripcslashes() is the inverse: it decodes the C-style escapes back to bytes.
$decoded = stripcslashes($escaped);
echo "roundtrip ok: ", ($decoded === $raw) ? "yes" : "no", "\n";
