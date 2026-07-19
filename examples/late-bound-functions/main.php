<?php

// PHP resolves a function call LATE: an undefined function only fatals when the call
// actually executes ("Fatal error: Uncaught Error: Call to undefined function ...()").
// A call to a curated, known-extension function name (apcu_*, opcache_*, xdebug_*,
// igbinary_*, frankenphp_*) compiles even though elephc never provides that extension —
// the call site lowers to a catchable \Error thrown with PHP's exact message, so code
// like Symfony's cache adapters (which guard these calls behind a cached
// extension_loaded() flag the compiler cannot fold away) still compiles and runs
// correctly, taking the fallback path instead of ever reaching the extension call.

class ApcuBackedCache
{
    private static ?bool $apcuSupported = null;

    private static function isSupported(): bool
    {
        // extension_loaded('apcu') always folds to false — elephc never ships APCu —
        // but caching the result in a static property (rather than calling it directly
        // in the guard below) hides that fact from the compiler's constant folder.
        return self::$apcuSupported ??= extension_loaded('apcu');
    }

    public static function fetch(string $key): string
    {
        if (self::isSupported()) {
            // Never reached on a target without APCu: this call site still had to
            // COMPILE, because the compiler cannot prove the guard is always false.
            apcu_exists($key);
            return "apcu:{$key}";
        }

        return "fallback:{$key}";
    }
}

echo ApcuBackedCache::fetch("user:42"), "\n";

// A direct, unguarded call to the same kind of function still compiles — and behaves
// exactly like real PHP: a catchable \Error, with the exact "Call to undefined
// function ...()" message, raised only once the call actually executes.
try {
    apcu_store("greeting", "hello");
    echo "unreachable\n";
} catch (\Error $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}

echo "done\n";
