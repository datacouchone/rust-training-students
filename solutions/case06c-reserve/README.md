# Challenge 06C — released reference solution

Released by the instructor on 2026-11-02. This independent crate contains the complete reference implementation and the same baseline and acceptance tests as the participant challenge. Your attempt in `production-labs/case06c-reserve` is unchanged.

From the repository root, retrieve just this solution without merging over your practice branch:

```sh
git fetch origin
git restore --source=origin/main -- solutions/case06c-reserve
cargo +nightly-2026-07-22 test --manifest-path solutions/case06c-reserve/Cargo.toml --offline --locked
```

The pinned toolchain must already be installed, as for the workshop labs. The solution has no external crate dependencies. No Docker or network service is needed for the tests.

## Read the five completed regions

- `attempt`: under the existing mutex guard, validate class and weight, check both capacity rules with checked arithmetic, then update one counter. Rejection leaves the snapshot unchanged.
- `wait_acquire`: attempt admission in a predicate loop, sleep only for Busy, and calculate remaining waiting time from one absolute deadline. Notifications do not grant a lease.
- `reconfigure`: validate the entire candidate against live usage before committing it, then wake waiters.
- `close`: mark the pool terminally closed and wake waiters; existing leases remain valid.
- `Lease::drop`: return the lease's recorded weight to its class and notify all heterogeneous waiters.

Read [the original challenge contract](../../production-labs/case06c-reserve/README.md) for the precise error and lifetime rules. The API blocks OS threads, has no fairness guarantee and does not revoke existing leases. A passing suite verifies the exercised cases; this teaching reference is not a production-readiness certification.
