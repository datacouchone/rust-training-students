# Day 1 extension bank

Choose an extension after the core outcome is working. The times below are
practice estimates, not extra scheduled sessions. Submit code plus the reasoning.
All tasks are independent, so nobody needs to finish an earlier extension first.

| ID | Topic and task | Time | Evidence to hand in |
|---|---|---:|---|
| E01 | Calculate Record padding before using size_of, align_of and offset_of | 10–15 min | Prediction, measured tuple and a reordered type |
| E02 | Encode Record to seven big-endian bytes without copying its padding | 15–20 min | Passing wire-byte tests and a decoding design |
| E03 | Classify repr(C) field order, custom Message size, Option<NonZeroU8>, Option<Box<u32>>, and nested String ABI | 10 min | Guarantee/observation/unsupported, with primary references |
| E04 | Restrict access to a logically live prefix, including malformed live_len | 10–15 min | Passing bounds tests and an initialization argument |
| E05 | Design a partially initialized array owner using MaybeUninit | 20–30 min | Initialized-count invariant, cleanup on early error, and panic discussion |
| E06 | Create a destructor that reads a borrow. Predict rejection when the referent drops first | 15–20 min | Compiler result, corrected ordering, marker/dropck explanation |
| E07 | Compare doubling growth with growth-by-one for 10,000 pushes | 15–20 min | Request counts, worst-case relocation work and memory trade-off |
| E08 | Validate Rust text for CString, including empty, Unicode and interior NUL | 10–15 min | Passing validation tests and pointer lifetime explanation |
| E09 | Draw get, overwrite-same-key and close with C/Rust ownership labels | 10–15 min | Lifetime diagram and explanation of the owned return value |
| E10 | Review exported checksum’s caller obligations | 15–20 min | # Safety contract and classification of null, dangling and empty inputs |
| E11 | Review MiniVec destruction when one element destructor panics | 20–30 min | Resource ownership map and a RawVec-style cleanup design |
| E12 | Review an FFI API that stores input pointers instead of copying | 20–30 min | Revised lifetime/ownership API and Send/Sync evidence required |

Four executable puzzles are in src/lib.rs: E01, E02, E04 and E08.
Discussion/design tasks should be recorded in your own REVIEW.md. E05 and E11
are advanced design work. Do not introduce unsafe implementation until you can
state the invariant and cleanup requirements.

```sh
cargo +stable check --all-targets --manifest-path exercises/day-01/Cargo.toml --offline
cargo +stable test --manifest-path exercises/day-01/Cargo.toml e02_wire_bytes --offline
```

Replace the test filter with e01_layout, e04_live_bounds or e08_c_strings.
The initial check compiles. Executing a TODO puzzle intentionally fails.
E01's recorded numeric expectations are for the course's normal desktop targets;
measure your target and discuss differences rather than changing types silently.

Never run deliberately invalid raw-pointer code natively. Predict it, use a
scratch copy under Miri, inspect the violated contract, and restore valid code.
A clean run covers tested executions rather than proving soundness.

Starter files remain puzzles after solution publication. Compare your work with
the separate answers/day-01/ directory when the instructor announces its release.
