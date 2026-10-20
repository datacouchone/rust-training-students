# Staged hints — attempt the stage before reading further

## A–D: initialized prefix

1. The allocation owns storage; len tracks the live values you must manage.
2. MaybeUninit::write initializes; assume_init_read transfers; assume_init_drop destroys.
3. A borrowed get needs index < len before pointer arithmetic. Write precedes len
   growth. Ownership transfer removes the slot from the live prefix. Cleanup uses
   that prefix, not capacity.

## E–G: one-shot ownership protocol

1. Observation and claiming are different. Two readers can observe the same state.
2. A compare_exchange can select one winner without replacing the representation.
3. Separate exclusive writer reservation, payload initialization, publication,
   exclusive consumer claim, and destruction. Decide which transition publishes
   other memory and which one must acquire it. No reset is permitted.

## H: independent atomic operation versus shared invariant

1. Unique IDs need an indivisible allocation. A consistent snapshot needs one
   coherent journal update, not merely individually protected fields.
2. AtomicUsize has a read-modify-write addition operation. Mutex guards delimit
   the critical section. The ticket's number is not the journal's publication flag.
3. Keep all journal changes under the same guard used by snapshot. Do not demand
   that ticket order equal completion order.
