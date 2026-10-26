<?php

class Job
{
    public string $name = 'Send invoice';
}

class PendingJobs extends SplObjectStorage
{
    private string $label = 'pending';

    public function label(): string
    {
        return $this->label;
    }
}

$queue = new PendingJobs();
$job = new Job();
$queue[$job] = ['attempts' => 1];

$restored = unserialize(serialize($queue));
echo $restored->label(), ': ', count($restored), "\n";
foreach ($restored as $job) {
    $info = $restored[$job];
    echo $job->name, ': ', $info['attempts'], " attempt(s)\n";
}
