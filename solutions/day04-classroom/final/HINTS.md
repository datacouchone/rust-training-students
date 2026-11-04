# Optional hints — read one level at a time

## 1. Follow ownership and time

A request's resource lifetime should agree with its work lifetime. Ask what still exists after the caller receives timeout. An empty buffer may still own a large allocation. A rejected operation must leave an inspectable state unchanged.

## 2. Find the boundaries

Inspect the first mutation in admission, the zero boundary in the retry counter, the scope of the routing guard, and the owner of the spawned attempt. Compare request start with each computed deadline. Read the complete telemetry predicate, including validity and the upper status bound.

## 3. Choose an API shape

Compute admission candidates locally and commit together. Use a conditional atomic update that cannot underflow. Copy the route decision before waiting upstream. A directly awaited I/O future can remain owned by its parent; if you keep spawned tasks, define cancellation and observed cleanup explicitly. Apply one absolute deadline to the entire retry loop. A prefix index can borrow path slices while searching. Retention policy runs after the previous buffer consumer finishes.

For telemetry, an iterator can select and accumulate without an intermediate vector. Measure before choosing a more complicated SIMD implementation. Keep the independent tests and honest counters.
