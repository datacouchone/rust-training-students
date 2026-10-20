# Hints: reveal only after an attempt

## A: options are a policy, not just syntax

1. Collect options across every summary attribute attached to the field.
2. Keep one record of flags and rename. Duplicate flags across separate
   attributes must be rejected just like duplicates in one attribute.
3. parse_nested_meta gives a source-aware error helper. Parse rename as LitStr.
   Validate the combined skip policy after collecting all attributes.

## B: labels collide after renaming

1. Two different Rust field identifiers can produce the same output label.
2. Check final labels of emitted fields; skipped fields emit nothing.
3. A set of label strings is enough. Report the second conflicting label.

## C: bounds belong to what you actually format

1. Requiring Debug on every type parameter rejects perfectly valid hidden data.
2. A field may be Option<T>, an array, a reference, or another composite type.
   Bound its actual type rather than guessing bounds from identifier spelling.
3. Clone the input generics, add predicates using make_where_clause, then use
   split_for_impl. Do not discard the user's where clause or copy defaults
   directly into impl generic declarations.

## D: redaction must happen before formatting

1. Formatting a secret and replacing its text afterwards is already too late.
2. Generate no field access or Debug call for a redacted field.
3. Treat renamed labels as string-literal data interpolated into tokens, not as
   a fragment of hand-built source code. Use absolute standard-library paths.
