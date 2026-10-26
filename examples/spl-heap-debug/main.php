<?php
// Inspect the physical priority-queue layout without consuming its elements.
$jobs = new SplPriorityQueue();
$jobs->insert('metrics', 1);
$jobs->insert('deploy', 5);
$jobs->insert('backup', 3);

print_r($jobs);

// Consume the same queue with both data and priority in each result.
$jobs->setExtractFlags(SplPriorityQueue::EXTR_BOTH);
while (!$jobs->isEmpty()) {
    $job = $jobs->extract();
    echo $job['data'], ': ', $job['priority'], "\n";
}
