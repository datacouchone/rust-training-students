# Hints — use one level, then try again

## Level 1 · follow ownership and observable state

Cancellation destroys a future, not bytes already consumed from a stream. A receipt and a charge describe one logical operation. A fast response can leave work behind. Admission and result waiting must refer to the same lifetime.

## Level 2 · identify the decision boundary

For framing, distinguish a field stored in the Decoder from a temporary local. For accounting, list every possible rejection before the first mutation. For replay, check both the identity and complete body. For async work, locate each spawned task and ask which owner cancels it. For rollout, distinguish no evidence from evidence of failure.

## Level 3 · API shapes

Use cancellation-safe incremental reads into retained state; construct candidate account values with checked arithmetic; use the receipt key before charging; keep one absolute Tokio Instant for the whole operation; use owned permits and an awaited future whose Drop releases local guards. A ledger lock is needed at commit, not during independent enrichment. Native comparison requires equal lengths as well as equal compared bytes.

These are implementation hints, not a complete solution. Add your own regression before changing another subsystem.
