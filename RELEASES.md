# Reference releases

Original starter files remain available. References are separate from participant attempts.

## 2026-11-02 — Day 3 challenge 06C

[Protected-capacity reference solution](solutions/case06c-reserve/README.md): completed admission, bounded waiting, live reconfiguration, closure and lease release, with the original challenge tests. Run it as an independent Cargo workspace; no edits to the challenge starter are required.

## 2026-11-02 — Day 3 main Lab 07

[Bounded-worker reference solution](solutions/case07-worker/README.md): completed `join_deadline` and `run_batch`, with the original tests in an independent Cargo workspace. The worker starter and challenge 07C remain unchanged.

## Earlier published references

[Lab 5 macros](solutions/lab5-macros/README.md) is also available in the solutions directory. Individual puzzle directories may contain previously released reference files.

## 2026-11-03 — Day 4 cumulative afternoon

Updated public Day 4 deck and [classroom setup](day04-classroom/README.md): cumulative memory and scan demos start at stage 0; focused 8B/8C labs and the three-repair final challenge retain their exercise tasks. Earlier worked examples through slide 16, original lab attempts and Day 5 materials are preserved.

## 2026-11-03 — SIMD depth revision

8C now contains three SIMD implementation tasks and focused tests. The final supplies gateway repairs and adds a SIMD event-record kernel, five transfer tests, workload-varied benchmarks and explicit service integration. Updated public slides and an eight-lane trace explain masks, widening, tails, layout and measurement. Original production labs and Day 5 are unchanged.

## 2026-11-04 — Day 5 worked demos through slide 10

[Completed framing and cancellation demonstrations](solutions/day05-through-slide10/README.md): the slide 8 complete-frame reader, slide 9 interrupted-header reader and slide 10 interrupted-body reader, with unchanged input drivers. Independent workspace; participant starters and later exercises remain untouched.

## 2026-11-04 — Day 5 references through slide 14

[Six completed demonstrations](solutions/day05-through-slide14/README.md) cover slides 8–13. [Lab 09 framing reference](solutions/case09-framing/README.md) completes the decoder task from slide 14 and retains all original tests. Four framing acceptance tests pass; two accounting tests intentionally remain failing until the later accounting lesson. Participant attempts and later exercises are unchanged.


## 2026-11-04 — Day 5 complete Lab 09 walkthrough (slide 18)

[Complete Lab 09 reference](solutions/case09-complete/README.md) combines cancellation-safe framing, transactional accounting and tenant cardinality checks. All six original acceptance tests and the baseline pass. Independent workspace; the framing-only reference and participant attempts remain unchanged.


## 2026-11-04 — Day 5 live incidents and native Linux task

[Runnable incident demos](day05-incident-demo/README.md) reproduce deadlock, task leakage and panic boundaries over localhost TCP, with separate original/repaired implementations. Slides 23–30 now include runnable captures, regressions, retry, review and release evidence. [Linux evidence worksheet](day05-incident-demo/LINUX-EXERCISE.md) uses GDB to launch its own child and capture all-thread backtraces; macOS uses sample. No original participant exercise is replaced.

Validation: five Rust regressions pass; all TCP comparisons, retry, release fixtures and macOS native capture were executed. Linux command construction, output validation and failure handling pass contract checks; a native Linux/GDB run remains unverified from the trainer's Mac. Captures are live thread evidence, not core dumps or tokio-console sessions.

## 2026-11-04 — classroom work and live diagnostic lab

- Preserve the saved Day 4 cumulative demo stages and Day 5 classroom repairs.
- Add slide 31, the quick Tokio Console/Linux core guide and launcher.
- Pin tokio-console 0.1.13 for compatibility with console-subscriber 0.4.1.
- Fix Docker workspace packaging for locked builds; compile with one Cargo job.
- Keep previously published solutions and participant instructions. The final hackathon remains an exercise, not a claimed completed solution.

Validation: Python syntax, six Cargo workspace manifests and the 38-slide deck were checked. The live viewer displayed twelve sleeping demo tasks; Docker captured and reopened a real Linux core before its image was removed for local storage cleanup. This publication does not claim every exercise test passes.

## 2026-11-04 — full lab reference solutions

Published Labs 1–9, all production cases, Lab 8B/8C, Day 4 final/challenge references and the Day 5 Project Relay solution under `solutions/`. All seven solution suites passed native tests, including all 23 original hackathon acceptance tests. See `solutions/README.md` and `solutions/VALIDATION.md`. Existing participant exercises remain separate.
