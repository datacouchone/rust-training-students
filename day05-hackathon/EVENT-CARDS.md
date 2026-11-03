# Incident updates — open at the time shown

## Minute 45 · the client never received the ACK

An accepted request may have committed before its response was lost. The client retries with the same tenant, identity and bytes. Demonstrate what the second request returns and what happened to accounting. Then change only the path while keeping that identity. Explain why “same ID” alone is insufficient.

## Minute 90 · the receipt ledger has reached its limit

The service has seen as many distinct request identities as it is allowed to remember. An old exact replay and a new identity arrive. Define and test each outcome. A proposed quick fix evicts an old receipt: explain the duplicate-effect risk before accepting that proposal.

## Minute 120 · the canary looks fast

Canary A: 10/10 successes, p95 2 ms, zero duplicates. Canary B: 995/1,000 successes, p95 40 ms, one duplicate effect. Candidate C: 990/1,000 successes, p95 50 ms, zero duplicates. Apply the exercise gate, then explain why meeting a numerical gate still needs compatibility and rollback evidence. No live deployment is performed.
