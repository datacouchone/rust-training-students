# Hints: open one stage after an attempt

## A and B: staging and preparation

1. Separate pending owners from committed owners. Merely changing a state must
   not publish anything.
2. An error can carry the original value or the original transaction. Borrowing
   or cloning that value is not the same contract.
3. Destructure or move the supplied fields when changing state. The new marker
   has no runtime state tag to update.

## C and D: committing a bounded history

1. Here head means oldest, unlike the core lab's next-write head.
2. While partially full, the next free physical slot depends on head plus len.
   Once full, replacement advances the oldest position.
3. Reserve the eviction vector before touching committed storage. Its maximum
   required length is bounded by the number of staged values.

## E and F: lending through physical wrap

1. A logical window can cross the physical array boundary. One contiguous
   mutable slice cannot represent that wrapped window without rearranging data.
2. The Window object borrows the journal and maps each local index separately.
   Its lifetime must come from this next call.
3. Check width, checked end position, and local index before modulo arithmetic.
   Capacity zero must never reach a remainder operation.
