<?php

// Usage is what makes the compiler plan the curl bridge and the managed native chain into
// the link, which is what this fixture exists to prove. It is NOT executed: an iOS build is
// a library, `elephc_init()` runs no top-level code, and an export that reached curl would
// be refused outright ("the cdylib boundary cannot prove process termination is
// unreachable"). The live transfer therefore runs on the host side of the boundary — see
// host.c and issue #873.
$handle = curl_init("https://example.com/");
curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
curl_exec($handle);

#[Export]
function ios_curl_link_smoke(): int
{
    return 0;
}
