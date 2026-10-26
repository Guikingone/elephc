<?php

$jobs = new SplObjectStorage();
$first = new stdClass();
$second = new stdClass();
$jobs->offsetSet($first, ['state' => 'queued']);
$jobs->offsetSet($second, ['state' => 'running']);

// Keep the diagnostic snapshot while the collection continues changing.
$debug = $jobs->__debugInfo();
$jobs->offsetSet($first, ['state' => 'done']);

foreach ($debug["\0SplObjectStorage\0storage"] as $entry) {
    echo $entry['inf']['state'], "\n";
}
