# Investigation report

Name / date:
Toolchain (`rustc +nightly-2026-07-22 --version`):

For each example, complete a row before and after repair.

| Example | Prediction | First useful diagnostic and line | Violated rule | Repair and why it works | Rerun evidence |
|---|---|---|---|---|---|
| 01 dangling | | | | | |
| 02 uninitialized | | | | | |
| 03 bounds | | | | | |
| 04 data race | | | | | |
| 05 lost update | | | | | |
| 06 publication (optional) | | | | | |

## Explain, not just pass

1. Why doesn't storing a raw pointer extend the owner's lifetime?
2. What is the difference between an allocated slot and an initialized element?
3. Why may a one-past address exist but not be read as an element?
4. Why does a scoped join not fix the conflicting writes in experiment 04?
5. Why doesn't SeqCst fix experiment 05? What must become indivisible?
6. Can a Mutex-based function be free of data races and still lose an update?
7. What connects the writer's release to the reader's acquire in experiment 06?
8. What can you claim after two counter tests pass under five seeds?
9. What does the native-C test cover that the Rust mock cannot establish?
10. Pick one remaining unsafe block and state every precondition needed here.

## Completion record

- A boundary tests:
- B initialization test:
- C locked counter:
- D atomic counter:
- Complete native suite (expected 2 + 4):
- Complete Miri suite (expected 2 + 4):
- Repaired examples run individually:
- Seeds tried:
- One behavior still not covered:
- One useful test to add and why it matters:
