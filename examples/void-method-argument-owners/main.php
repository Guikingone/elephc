<?php

final class SnapshotReceiver
{
    public int $received = 0;

    public function __unserialize(array $data): void
    {
        $this->received = count($data[0]);
    }
}

function restore(mixed $receiver, mixed $job): void
{
    $receiver->__unserialize([[$job], ['state' => 'ready']]);
}

$job = new stdClass();
$metadata = new SplObjectStorage();
$metadata[$job] = 'ready';
$receiver = new SnapshotReceiver();
restore($receiver, $job);
echo 'Restored ', $receiver->received, ' item (', count($metadata), " metadata entry)\n";
