# Final — SIMD telemetry investigation (90 minutes)

Transfer the 8C kernel to real event records, measure it, and integrate it with the
provided gateway. The non-SIMD gateway repairs are supplied: admission, retry
accounting, route indexing, lock scope, deadline/cancellation and buffer retention.
Run their examples and explain observed outcomes briefly; your implementation time
belongs to SIMD. Original service acceptance tests remain unchanged.

## Work and timing

- 10 min: run baseline/acceptance and healthy/slow/burst examples. The service tests
  pass; five new SIMD tests initially fail at TODOs. Record that distinction.
- 35 min: implement `summarize_simd` in `src/telemetry.rs`: full validity/latency/
  status mask, match count, widened sum, and all remaining events.
- 20 min: compare scalar and SIMD across sizes, two error distributions and two
  latency thresholds. Include AoS field packing in SIMD timing.
- 15 min: run healthy-simd, preserve all service tests, inspect assembly or attempt
  another width. If changing width, change bulk/step/tail consistently and retest.
- 10 min: explain the result and recommend scalar, SIMD or a measured dispatch rule.

## Start from the participant repository root

```sh
python3 day04-classroom/final/run.py baseline
python3 day04-classroom/final/run.py healthy
python3 day04-classroom/final/run.py slow
python3 day04-classroom/final/run.py burst
python3 day04-classroom/final/run.py simd-test
```

After implementing the SIMD tasks:

```sh
python3 day04-classroom/final/run.py acceptance
python3 day04-classroom/final/run.py simd-bench
python3 day04-classroom/final/run.py healthy-simd
```

`acceptance` runs the 12 original service assertions plus five SIMD tests. Use
`cargo +nightly-2026-07-22 test --manifest-path day04-classroom/final/Cargo.toml --offline --locked --test acceptance`
to check supplied service behavior independently of the SIMD TODOs.

## What differs from 8C

8C loaded contiguous columns. Here `Event { latency: u32, status: u16, valid: bool }`
is a row record. Supplied `from_array(from_fn(...))` packing reads fields from eight
records. Build your vector predicate and reduction on those vectors, then handle
the tail. Status is widened from u16 when packed. The result must match independent
`summarize` for valid events with latency >= threshold and status in 500..=599.
Keep the scalar oracle unchanged. The fixture bounds totals to fit u64; widen each
u32 lane before reduction. No unsafe pointer loads are needed.

Default service commands use scalar telemetry so the service can run before the
SIMD task is complete. `healthy-simd` explicitly selects your SIMD implementation
for the summary of actual responses. That small 24-event batch demonstrates
integration, not a credible service latency speedup. The dedicated benchmark uses
7/128/4099/100003 events, seven samples, rotating order, and ns per scan.

## Evidence to submit

Code implementing actual vector comparisons/masks/reduction; all 17 acceptance
tests; raw benchmark output; CPU/compiler/build settings; median ns per identical
case; a case where SIMD loses or a measured limitation; and the healthy-simd output.
Do not delegate the SIMD task to summarize merely to pass tests. No universal
speedup or size threshold is required. Optional: compare four/eight lanes or add
an explicit SoA conversion and measure both conversion+scan and scan-only costs.

## Service boundaries

This is a local closed-loop TCP fixture with two Tokio workers, not HTTP or a
production benchmark. Healthy/slow/burst preserve their original inputs, outcome
counts and counters. All sockets are loopback with dynamic ports. No Docker.
Do not infer service p95 improvement from the isolated SIMD microbenchmark.
See [the SIMD guide](../SIMD-GUIDE.md) for lane arithmetic and hardware limits.
