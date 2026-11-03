# SIMD hints — use one level at a time

1. Concept: one lane represents one Event. Every predicate condition must hold.
   Masked-out lanes contribute zero. Reduction combines lanes; the tail is separate.
2. API shape: simd_ge/simd_le produce masks; & combines them; select chooses values;
   to_bitmask/count_ones counts matches; cast::<u64>() must precede reduce_sum.
3. Debugging: write the expected eight Boolean results on paper. Inspect lane order,
   status <=599, nulls, the conversion position, and bulk..events.len(). Keep
   summarize independent. Include field packing when timing your SIMD version.
