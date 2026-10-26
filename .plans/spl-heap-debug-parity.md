# SPL heap debug views: unchanged bug69264

Pinned PHP oracle and full PHPT corpus remain those in main-integration-1806.md. No publication.

## Confirmed on M16

- Per-class reduction under .artifacts/integration/main-m16/bug69264-reduction: SplFileInfo, SplDoublyLinkedList, SplObjectStorage, MultipleIterator, ArrayObject and ArrayIterator compile. SplMaxHeap and SplPriorityQueue fail with AssocArray -> Array(Mixed) backend conversion errors.
- Both __debugInfo declarations in builtin_spl_classes/heaps.rs use Some(array_type()), unlike other SPL views' php_array_contract_type(). Do not erase keys or pretend hash and packed array have the same physical representation.
- Existing heap_debug_info_body returns only data; priority_debug_info_body returns flags/data/priorities. These schemas disagree with pinned Zend.
- Executed PHP serialized views, before and after extract, are retained under main-m16/heap-debug-oracles. Heap views expose private keys flags / isCorrupted / heap from SplHeap, priority views from SplPriorityQueue. Queue heap entries pair data and priority. Heap-array order is physical binary-heap order, not a sorted list or insertion order.

## Required work

- [ ] Trace return-contract helper and heap method callers before edits.
- [ ] Correct the declared PHP array contract separately from backing storage.
- [ ] Reconcile binary heap insertion/removal and exceptional comparison corruption so a snapshot reflects actual current heap order.
- [ ] Preserve ordinary user properties and visibility mangling without exposing implementation storage.
- [ ] Add direct getter, inherited native var_dump/print_r, non-empty/extracted state and lifetime regressions; keep php-src PHPT unchanged.
- [ ] Verify all supported targets and focused heap regressions before broad replay.

A six-class assembly success is not a runtime/parity pass. Do not close bug69264 merely after fixing its return-type compiler error.

## Pinned Zend algorithm inspected

Oracle ext/spl/spl_heap.c:269-295 inserts with a write lock, compares parent vs incoming and sifts upward, increments count even if compare leaves an exception, clears lock, marks corruption, then copies the incoming element. At 307-352 removal saves the root, decrements count, moves the former tail down, compares right child vs left only when present, then bottom vs chosen child; it marks corruption after comparisons and completes the final copy. Throwing directly from a generated PHP comparison before this bookkeeping would differ from Zend. Separate internal heap flags (CORRUPTED=1 / WRITE_LOCKED=2) from the user-facing queue extraction flags. Clone copies both physical element order and flags (356-375).

## M18c implementation and proof

Heap and queue metadata is split into state, mutation, readers and debug snapshot modules. Arrays store physical binary-heap order; private state tracks live count, write lock and pending comparison errors. Getter schemas preserve user members and use Zend private flags/isCorrupted/heap keys. Extraction selects queue flags after comparisons. setExtractFlags masks its input. Current/key bypass user count/isEmpty overrides and empty keys are -1.

M18c CLI 82a279be03206000378a2420dcbe105defc46148ad49b68cd3da1e2c4b3ddd03 matches 14/15 custom fixtures in both IR modes with clean heaps (28/30 executions). Unchanged bug69264 now passes: after the heap contract repair it exposed an unretained MultipleIterator::__debugInfo symbol; that getter now uses the PHP array contract, the SplObjectStorage private storage key and obj/inf rows, and is whitelisted for lazy body retention. All filtered results remain complete=false.

The remaining next_destructor fixture prints drop:1 instead of PHP drop:2, and leaks 272 bytes. Owned method receiver guards are independently fixed and validated against latest main 4fdabfb8b1cd0b3bc58e789c846cf5970e86490e in elephc-method-receiver-main-baseline, not yet integrated. Its receiver-fix-1 checkpoint passes all 18 native controls; Rust and target verification pending.

The counter/order defect requires releasing the root before decrementing count and before comparison callbacks. Do not fake count only during destructors: that would leave callback order wrong. Do not call destructors manually while retaining a strong root: weak references/resurrection would differ.
