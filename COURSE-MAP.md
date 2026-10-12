# Concept-by-concept puzzle map

| Concept | Together | Your turn | Advanced extension |
|---|---|---|---|
| [Values, functions and borrowing](lessons/p00_basics.md) | Return the UTF-8 byte length of borrowed text. | Return a new owned greeting without taking ownership of the input text. | prereq-check |
| [Alignment and padding](lessons/p01_layout.md) | Compute padding before a field for a nonzero power-of-two alignment. | Compute the final padded size for a nonzero power-of-two struct alignment. | lab1-layout |
| [Safe interfaces over unsafe operations](lessons/p02_unsafe.md) | Return a checked reference using pointer arithmetic; return None outside bounds. | Swap two elements through pointers only after checking both indices; otherwise return false. | lab2-minivec |
| [Pointer and length at a foreign boundary](lessons/p03_ffi.md) | Wrap foreign_sum with a safe slice API and a justified unsafe call. | Wrap foreign_max with the same pointer/length contract, including empty input. | lab3-ffi |
| [State transitions encoded in types](lessons/p04_typestate.md) | Implement Closed -> Open without losing the door's name. | Implement Open -> Closed; verify that walking through a closed door cannot compile. | lab4-typestate |
| [Macros and expression evaluation](lessons/p05_macros.md) | Expand make_pair! for a Copy expression while evaluating that expression only once. | Expand repeat_vec! for Clone values; evaluate the value expression only once. | lab5-derive-macro |
| [Atomic counters and publication](lessons/p06_atomics.md) | Return the previous value while incrementing an atomic ID counter; justify the ordering. | Use a ready flag to publish and observe prior writes; name the required ordering pair. | lab6-atomics |
| [Future polling and wakeups](lessons/p07_future.md) | Make Countdown return Pending with a wakeup while remaining > 0, then Ready. | Make AddLater return Pending once with a wakeup, then Ready with its sum. | lab7-executor |
| [Chunk processing and correctness before timing](lessons/p08_performance.md) | Sum u32 values into u64 in chunks of four, including all tail elements. | Count values at or above a threshold in one pass without allocating an intermediate Vec. | lab8-simd-perf |
| [Parse errors and framed input](lessons/p09_debugging.md) | Parse name=value into a Metric; reject missing names, separators, and non-finite values. | Decode one complete big-endian u32-length-prefixed UTF-8 frame; reject malformed frames. | lab9-core-dump |
| [Industry practices](lessons/p10_industry.md) | Unsafe PR review | Adoption pilot | Module 10 discussion |

The stable mini-puzzles cover a focused entry point for each module. The nine advanced labs retain the wider course scope: memory representations, MiniVec, actual C linkage, GATs, procedural macros, full atomics/executor implementations, SIMD/arenas, and production debugging. Do not treat a single mini-puzzle as completion of the entire module.

## Entry point for this advanced course

Attendees already know Rust: begin delivery with P01 / Module 1, using a
struct-layout prediction. P00 is optional prerequisite practice. Keep explanations short, ask for a prediction, then
run a demo and use the lab to practise reasoning. Fast finishers use the
lab's stretch goals; allow others to finish the core contract and debrief.
