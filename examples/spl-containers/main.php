<?php

$list = new SplDoublyLinkedList();
$list->push("alpha");
$list->push("beta");
$list->unshift("start");

foreach ($list as $index => $value) {
    echo $index;
    echo ": ";
    echo $value;
    echo "\n";
}

$queue = new SplQueue();
$first = $argc === 1 ? "first" : 1;
$queue->enqueue($first);
unset($first);
$queue->enqueue("second");
echo "queue: ";
echo $queue->dequeue();
echo "\n";
echo print_r($queue, true);

$stack = new SplStack();
$stack->push(10);
$stack->push(20);
echo "stack: ";
echo $stack->pop();
echo "\n";

$fixed = new SplFixedArray(2);
$fixed[0] = "left";
$fixed[1] = "right";
echo "fixed: ";
echo $fixed[0];
echo ", ";
echo $fixed[1];
echo "\n";

$max = new SplMaxHeap();
$min = new SplMinHeap();
foreach ([3, 1, 5, 2] as $value) {
    $max->insert($value);
    $min->insert($value);
}

echo "max heap: ";
while (!$max->isEmpty()) {
    echo $max->extract();
}
echo "\n";

echo "min heap: ";
foreach ($min as $value) {
    echo $value;
}
echo "\n";

$priority = new SplPriorityQueue();
$priority->insert("low", 1);
$priority->insert("high", 5);
$priority->insert("mid", 3);
$priority->setExtractFlags(SplPriorityQueue::EXTR_BOTH);
$task = $priority->extract();
echo "priority: ";
echo $task["data"];
echo " ";
echo $task["priority"];
echo "\n";

class Job {}

$left = new Job();
$right = new Job();
$storage = new SplObjectStorage();
$storage->offsetSet($left, "left");
$storage[$right] = "right";

echo "object storage: ";
foreach ($storage as $job) {
    echo $storage[$job];
    echo " ";
}
echo count($storage);
echo "\n";

// A queue subclass keeps its native list state alongside application properties.
class WorkQueue extends SplQueue {
    public string $name = "mail";

    public function __construct(string $first) {
        $this->enqueue($first);
    }
}

$work = new WorkQueue("welcome");
echo $work->name;
echo ": ";
echo $work->dequeue();
echo "\n";

$submit = $work->enqueue(...);
$submit("follow-up");
echo "next: ";
echo $work->dequeue();
echo "\n";
