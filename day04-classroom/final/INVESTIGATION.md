# SIMD evidence record

CPU / OS / rustc -Vv / build flags / commit:

1. Run healthy, slow and burst with supplied scalar telemetry. Record outcomes,
   active attempts and retained capacity. Explain each result in one sentence.
2. Predict one eight-event mask, its count and sum before running your SIMD kernel.
3. Explain AoS packing, widening and the tail. Show the test that catches each bug.
4. Record medians of seven samples for identical size/distribution/threshold cases.

| Rows | Distribution | Threshold | Scalar ns/scan | SIMD ns/scan | Explanation |
|---|---|---|---|---|---|
| | | | | | |

5. Run healthy-simd. Separate correctness/integration from timing evidence.
6. Inspect emitted assembly or compare another lane width. State exactly what was
   observed on this target; do not claim eight lanes imply one instruction.
7. Recommend a candidate and one held-out workload to validate the decision.
