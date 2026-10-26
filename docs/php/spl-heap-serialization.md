# SPL heap serialization

`SplHeap::__serialize()` and `SplPriorityQueue::__serialize()` return PHP's indexed
pair of user properties and native heap state. `SplMaxHeap` and `SplMinHeap` inherit
the hook from `SplHeap`. Calling `serialize($heap)` uses that hook automatically.

The first array contains initialized user properties with PHP visibility names,
including mangled protected and private names. Compiler backing fields are omitted.
The second array contains `flags` followed by `heap_elements`. Regular heaps export
flags `0`; priority queues export their extraction flags and each element as
`['data' => ..., 'priority' => ...]`. Elements retain physical binary heap order.

Serialization leaves the heap intact. A corrupted heap throws `RuntimeException`.
Serializing during a comparison or another active mutation throws
`RuntimeException` with `Cannot serialize heap while it is being modified.`

See [the queue example](../../examples/spl-heap-serialization/main.php).

The native `__unserialize()` loading protocol remains an open compliance gate.
