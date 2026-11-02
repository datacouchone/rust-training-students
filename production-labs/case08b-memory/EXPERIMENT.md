# Performance decision record

- Engineer / date / commit:
- CPU, OS, compiler (`rustc -Vv`), build flags:
- Workload size, distribution, thresholds and exact commands:
- Hypothesis and predicted bottleneck:
- Correctness evidence (test names and differential outputs):
- Baseline and candidate raw CSV/profile paths:

| Workload | Implementation | Samples / iterations | Median | Min–max | Memory / build cost | Interpretation |
|---|---|---|---|---|---|---|
| | | | | | | |

- Timing boundaries, excluded setup and known measurement overhead:
- One plausible optimization rejected, with evidence:
- Held-out input and result:
- Ship / limit / reject decision, break-even or dispatch policy:
- Remaining production risks and next measurement:

Do not replace failed correctness checks with a timing claim. A synthetic CPU measurement does not establish service p99 latency or a vendor's performance result.
