# P10 — Production practices: review puzzles

## Together (8 minutes)
A PR adds an unsafe fast path. One benchmark reports a speedup; tests pass,
but the PR has no safety argument or comparison with the safe implementation.
What evidence should a reviewer ask for before making a release decision?
Identify correctness evidence separately from performance evidence.

## Your turn (12 minutes)
A team proposes rewriting a working service in Rust to eliminate every class
of security bug. In pairs, propose a small adoption pilot, name the classes
of bugs Rust helps prevent and those it does not, and define measurable
success and rollback criteria. List the production checks you would require.

Done: present your decision, assumptions, evidence needed, and one remaining
risk. This is a discussion puzzle, not a new syntax exercise or universal
policy rule. Vendor-specific numerical claims require dated primary sources.
