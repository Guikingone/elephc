<?php

class Jobs extends SplPriorityQueue
{
    public string $name = 'ready';
}

$jobs = new Jobs();
$jobs->insert('background', 1);
$jobs->insert('urgent', 10);
$jobs->setExtractFlags(SplPriorityQueue::EXTR_BOTH);

$state = $jobs->__serialize();
echo 'Queue: ', $state[0]['name'], "\n";
echo 'Flags: ', $state[1]['flags'], "\n";
echo 'Jobs: ', count($state[1]['heap_elements']), "\n";
echo serialize($jobs), "\n";

$restored = new Jobs();
$restored->__unserialize($state);
echo 'Restored: ', $restored->name, "\n";
while (!$restored->isEmpty()) {
    $job = $restored->extract();
    echo $job['data'], ':', $job['priority'], "\n";
}
