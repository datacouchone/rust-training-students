# Predict first: the boundary traps

Classify each as: valid, compile error, returned error, or forbidden unsound
approach. These are discussion fragments, not a file to run wholesale.

1. A key contains bytes `[0xff, 0]`. Which error wins? Why does the ordering
   matter for a stable public API?
2. A stored value is `""`. Does lookup mean missing? What about an empty key?
3. A value from get_str is printed after db.put replaces its key. Should safe
   Rust accept that program? Contrast printing before the put (last use).
4. A key is a temporary Vec destroyed at the end of an inner block, but the
   database lives on. Should a result borrowed from that lookup still work?
5. A transform panics after reading the old value. May the wrapper leave a
   placeholder in the database, or silently turn the panic into success?
6. A transform returns `"new\0value"`. Should the old value already have been
   replaced when the error is returned?
7. A transform consumes a captured String by returning it. Is FnOnce enough,
   or must the API require a repeatable callback?
8. Someone uses CString::from_raw on legacy_get's pointer because it is a
   terminated string. Did the header transfer ownership to Rust?
9. Someone makes an owned String locally and returns a reference to it. Does
   giving the return type a database lifetime extend that local allocation?
10. Someone uses an unsafe lifetime cast to make a borrowed result 'static.
    Which operation can now invalidate that apparently permanent reference?
11. Miri passes with the Rust mock. Has the native C implementation been
    checked by Miri? Which evidence comes from the normal native test run?

Deliverable: defend your two method bodies, not just a green test summary.
