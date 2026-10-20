# Hints: attempt each puzzle before opening its next hint

## A — checked read

1. A slice already knows its initialized length; use that information.
2. `get` returns an optional reference without panicking on an invalid index.
3. Turn the optional reference into an optional copied u32.

## B — initialization

1. Allocating storage and initializing a value are separate operations.
2. `MaybeUninit::write` initializes the slot; `assume_init` consumes it as T.
3. The write must occur on every path reaching the unsafe read. Explain that fact.

## C — locked increment

1. Two individually protected accesses can still leave a gap between them.
2. Keep a mutable lock guard alive for the entire increment.
3. Dereference the guard to update its protected usize; release on scope exit.

## D — atomic increment

1. A load followed by a store is two operations, even if both are SeqCst.
2. Look for the atomic read-modify-write addition method.
3. Its return value is the old count. No separate payload is being published here.

## Optional publication

1. The flag must connect the payload write to the later payload read.
2. A writer publishes prior operations; a reader must acquire that publication.
3. Identify the store the reader observed. Relaxed on either side loses the
   intended synchronization for the non-atomic payload in this example.
