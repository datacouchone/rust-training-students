# Published lab solutions

Worked reference implementations, published after the exercises at the trainer's request. Keep your own work and use these separate directories for comparison.

## Start here

| Lab | Solution |
|---|---|
| Day 5 final hackathon — Project Relay | [day05-hackathon](day05-hackathon/) |
| Lab 8B — allocation and retention | [day04-classroom/8b](day04-classroom/8b/) |
| Lab 8C — SIMD masks, reductions and tails | [day04-classroom/8c](day04-classroom/8c/) |
| Day 4 final lab — server and SIMD telemetry | [day04-classroom/final](day04-classroom/final/) |
| Day 4 production challenge | [day04-challenge](day04-challenge/) |
| Production case studies 06–09 | [production-labs](production-labs/) |
| Numbered labs 1–9 | [labs](labs/) |

The Day 5 final hackathon should pass **all 23 original acceptance tests**:

```sh
git pull --ff-only
python3 solutions/day05-hackathon/run.py acceptance
```

Run every newly published solution suite:

```sh
python3 solutions/run_all.py
```

Each solution README includes focused commands. Use the pinned nightly and keep Cargo.lock. Commands use one compilation job to reduce memory pressure. If a fresh machine lacks cached dependencies, run the documented Cargo test command once without `--offline`.

## Previously published classroom checkpoints

The existing `case06c-reserve`, `case07-worker`, `case08a-profile`, `case09-framing`, `case09-complete`, `day05-through-slide10`, `day05-through-slide14` and `lab5-macros` directories remain available. Some are intentionally partial classroom checkpoints; use the complete solutions above for final review.

See [VALIDATION.md](VALIDATION.md) for the actual publication checks. Passing tests is evidence for the supplied cases, not proof of all schedules, unsafe-code soundness, performance, or production durability.
