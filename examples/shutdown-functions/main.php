<?php

// register_shutdown_function() queues cleanup work that runs after the script's
// normal output, in registration order, even if the script ends via exit()/die().

function processOrder(string $id, float $total): void
{
    echo "Processing order {$id} (\${$total})\n";

    register_shutdown_function(function () use ($id) {
        echo "Order {$id}: releasing lock\n";
    });

    if ($total > 1000.0) {
        echo "Order {$id}: flagged for manual review\n";
        exit(1);
    }

    echo "Order {$id}: confirmed\n";
}

register_shutdown_function(function () {
    echo "Shutting down: closing database connection\n";
});

processOrder("A-100", 250.0);
processOrder("B-200", 1500.0);

echo "unreachable\n";
