<?php

class Batch
{
    public mixed $owner = null;

    public function __destruct()
    {
        echo "Batch released\n";
    }
}

function prepareBatch(): void
{
    $batch = new Batch();
    $batch->owner = $batch;
}

prepareBatch();
echo "Work finished\n";
