<?php

$handle = curl_init("https://example.com/");
curl_setopt($handle, CURLOPT_RETURNTRANSFER, true);
curl_exec($handle);

#[Export]
function ios_curl_link_smoke(): int
{
    return 0;
}
