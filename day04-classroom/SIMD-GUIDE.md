# SIMD: one predicate across several rows

## Meaning before syntax

SIMD means single instruction, multiple data. A vector value describes the same
operation on several lanes. It is not eight threads, and it does not promise an
8x speedup. Portable Rust vectors may require several machine instructions.
The compiler may also vectorize scalar source. We compare optimized release builds.
Our pinned nightly enables the experimental portable_simd API; it is not a stable
Rust API. This format teaches column/validity ideas; it is not Arrow IPC.

## Work one complete group by hand

Query: latency >= 100 AND status in 500..=599 AND valid.

| Lane | Latency | Status | Valid | Latency passes | Status passes | Selected value |
|---|---|---|---|---|---|---|
| 0 | 10 | 503 | yes | no | yes | 0 |
| 1 | 100 | 200 | yes | yes | no | 0 |
| 2 | 200 | 503 | no | yes | yes | 0 |
| 3 | 300 | 600 | yes | yes | no | 0 |
| 4 | 400 | 599 | yes | yes | yes | 400 |
| 5 | 500 | 500 | yes | yes | yes | 500 |
| 6 | 600 | 503 | yes | yes | yes | 600 |
| 7 | 700 | 503 | no | yes | yes | 0 |

Predict count=3, sum=1500. The mask printed as binary is 01110000: lane 0 is the
RIGHTMOST bit. A table written lane 0 first has the opposite visual direction.

```sh
cargo +nightly-2026-07-22 run --manifest-path day04-classroom/demos/8c/Cargo.toml --bin explain-lanes --offline --locked
```

## How the Rust operations connect

`Simd<u32,8>` holds eight unsigned 32-bit lane values. `from_slice` loads a full
group from contiguous column values; supply eight accessible values. It does not
require the caller's slice to have vector alignment. Do not cast a slice pointer
to a vector pointer to imitate this safe operation.

`splat(100)` repeats 100 into every lane. `simd_ge` compares corresponding lanes;
it produces a mask, not one bool. `Mask<i32,8>` has eight Boolean decisions; i32
is the mask element type paired with u32 vectors, not an array of status values.
`&` combines decisions lane by lane. Scalar `&&` cannot combine vector masks.
A validity bitmap stores eight row-presence bits in one byte. The validity mask
expands these into eight decisions. A nonzero value in a null row must be ignored.

`mask.select(latency, zero)` keeps matching values and replaces the rest with zero.
`to_bitmask().count_ones()` counts selected lanes. `reduce_sum()` combines values
across lanes into one result. Selection and reduction are different operations.

## Widen before reducing

Eight u32::MAX values total 34,359,738,360, which cannot fit in u32. Convert EACH
selected lane to u64 before reduce_sum. Casting a wrapped u32 sum afterward cannot
recover the lost value. Eight u32 values fit in u64; checked addition protects the
running total across groups in the columnar lab. Widening can change the machine
instruction sequence: inspect generated code rather than assuming a free cast.

## Tails and bitmap indexing

For 11 rows and width 8: bulk = 11 / 8 * 8 = 8. Process indices 0..8 as one vector;
then indices 8,9,10 separately. Row 10 uses bitmap byte 10/8=1 and bit 10%8=2.
Restarting a tail-local index at zero reads the WRONG validity bits. For 7 rows
there is no full vector; all seven rows belong to the tail. Empty input has neither.

## Layout and hardware

8C stores separate contiguous latency and status columns (SoA). The final stores
Event records containing all fields (AoS). Its supplied field-packing code reads
individual records to construct vectors. Include that work in timing: reusing the
same arithmetic does not make the memory access identical.

Unrolled scalar uses several accumulators but is not explicitly a SIMD program.
Whether either implementation uses vector instructions depends on compiler,
target and optimizations. Inspect actual output, then report what you observe:

```sh
cargo +nightly-2026-07-22 rustc --release --manifest-path day04-classroom/labs/8c/Cargo.toml --lib --offline --locked -- --emit=asm
```

After completing the TODOs, inspect the generated `.s` file under that crate's
`target/release/deps/`. Search for scan_simd and the mask/reduction helpers. Tiny
inputs emphasize setup/tails; larger inputs add memory traffic. Selectivity and
null density can change work and branch behavior. Treat these as hypotheses.

## Sources

- [Rust portable SIMD](https://doc.rust-lang.org/core/simd/index.html)
- [Rust Simd methods](https://doc.rust-lang.org/nightly/core/simd/struct.Simd.html)
- [LLVM auto-vectorization](https://llvm.org/docs/Vectorizers.html)
