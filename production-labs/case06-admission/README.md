# Admission control for a multi-tenant gateway

Delivery Day 3 · Production case 06

A burst of uploads must respect both a process memory budget and individual tenant budgets. Tenant rejection, decode failure and task cancellation must release any reservation.

## Evidence and adaptation

Amazon describes early admission and load shedding under overload. This teaching implementation uses weighted atomic reservations to make rollback and ownership obligations observable.

- [Source 1](https://builder.aws.com/content/3Eun1EEyX6p2e3VYNyRLSJzLuMV/using-load-shedding-to-avoid-overload)
- [Source 2](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)

This is an original classroom implementation. Company sources establish the motivating problem or technique; they do not endorse this code or establish production readiness. Sources checked 2026-10-31.

## Your contract

1. Use a CAS loop for checked weighted reservations; bound every successful reservation by the configured limit.
2. Acquire global then tenant budget and roll back global admission when tenant admission fails.
3. Make a non-Clone lease own release on ordinary return, unwind and cancellation. Justify orderings operation by operation.

Start with `src/lib.rs`; acceptance scenarios are in `tests/acceptance.rs`.
Implementation sequence: **debit -> release -> reserve**.

## Run from this production-labs directory

```sh
cargo +nightly-2026-07-22 test -p case06-admission --test baseline --offline --locked
cargo +nightly-2026-07-22 test -p case06-admission --test acceptance --offline --locked
```

The baseline checks supplied infrastructure only. Acceptance failures are expected initially. Work on one named test at a time; passing baseline or compilation does not complete the case.

## Adversarial acceptance

- Simultaneous contenders held at barriers.
- Exactly-full, zero-byte, usize::MAX and overflow requests.
- Two tenants sharing global capacity, rejected tenant rollback and panic cleanup.

## Advanced transfer

Connect leases to case07 jobs and release on timeout. Compare contention with Mutex and sharded budgets; formulate a fairness policy separately from capacity safety.

This transfer is an open design/implementation task, not an already supplied acceptance-tested extension. Specify new acceptance evidence before coding.

## Finish with evidence

Submit your change, a previously failing case, a passing regression check, the invariant or ownership argument, and one remaining limitation. Explain a plausible fix you rejected and why. Keep the API and tests; do not delete assertions or relax the contract to pass.

## Boundaries

Reservations account numbers and do not publish payload memory. Global and tenant counters are individually atomic, not an atomic pair snapshot. No fairness guarantee. Forgetting a lease leaks capacity; untrusted callers must not control lease ownership.
