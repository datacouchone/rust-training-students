# Day 5 — diagnose, repair and defend the release

Open the [participant deck](presentations/day-05/index.html). The coach deck remains private.

Morning: investigate [Lab 09](production-labs/case09-incident/README.md) in two parts — cancellation-safe framing and transactional accounting. Nine separate trainer demos under `day05-demos/` illustrate the mechanisms without changing the participant lab. Then connect structured evidence, async task ownership and release review.

Afternoon: [Project Relay](day05-hackathon/README.md), a **150-minute hackathon in teams of two**. Reserve 14:00–16:30, then group review; adjust the start time to the actual session. One driver and one evidence reviewer, swapping at minute 50. It integrates the central topics from Days 1–5 through six repair stations, three incident updates, measurements and a release handoff. Add presentation time if the class is large.

```sh
python3 day05-hackathon/run.py baseline
python3 day05-hackathon/run.py probe load
python3 day05-hackathon/run.py probe replay
python3 day05-hackathon/run.py acceptance
```

Baseline passes; initial acceptance failures are deliberate. Follow the challenge contract and preserve fixtures/tests. Each pair adds two adversarial regressions and explains a change written by the other partner. The reference stays private. The small native workload is not a production SLO test.

The native TCP scenarios and deterministic duplex regressions require no Docker. Core capture, Docker incidents and live deployment remain deferred. The earlier lab and previous-day practice files remain separate.
