# Defend the handoff

Classify each change: valid, compile-time rejection, logical bug, memory UB,
or insufficient evidence. A passing test is not the complete explanation.

1. Batch has capacity 8 and length 2. get(2) is inside the allocation: may it return &T?
2. Compute base.add(index) first, then check index < len. Is usize::MAX harmless?
3. pop reads T but does not shorten len. Which owner later destroys the same value?
4. A rejected push drops value and returns a cloned replacement. Is that the API contract?
5. After pop, the old bits remain. Should we clear the slot with zeroes before reusing it?
6. A publisher stores READY before initializing T. Does a Release store rescue this order?
7. Both a ready check and a taken store use SeqCst, but are separate operations. Can two takers win?
8. A failed try_take changes EMPTY into TAKEN. Can a later publisher still succeed as required?
9. A reader performs an Acquire load, but the READY store is Relaxed. What publishes the payload?
10. The main thread joins the producer before taking. Can the join mask a missing publication edge?
11. Drop always destroys slot. What happens after a successful transfer? What about EMPTY?
12. Require T: Sync even though the API only transfers T. What happens to Cell<u32>?
13. Hand an Rc<u32> through OneShot using a manual unconditional Sync implementation. Is that justified?
14. Allocate a ticket with load plus store, both SeqCst. Is every caller guaranteed a unique ID?
15. Hold one guard for completed, another for byte sum, another for entries. Can snapshots be inconsistent?
16. IDs appear in completion order [2, 0, 1]. Is that automatically an error?
17. Reset TAKEN to EMPTY so the object can be reused. Does the one-shot safety argument still apply?
18. The three compile-fail doctests pass with TODO bodies. Is the implementation complete?
19. All tests pass on one Miri seed. Are other callers, values and schedules proved safe?
20. A Rust mock of a foreign library passes Miri. Did Miri execute the native C library?
