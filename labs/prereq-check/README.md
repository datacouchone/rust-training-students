# prereq-check — Course Prerequisite Self-Check

This tiny crate is the self-administered gate for *Advanced Rust: Systems,
Concurrency & Internals* (see this edition's course map). The course's
stated prerequisite is:

> Minimum 1–2 years of production experience writing standard Rust
> applications... thorough, practical understanding of Rust's ownership,
> borrowing, and standard lifetime rules.

This crate turns that sentence into something you can actually test yourself
against, in about the same amount of time it takes to read this file.

## The 30-minute rule

Set a timer for **30 minutes**. Then run the challenge build (below) and fix
all four broken items — **with no lookups**: no docs.rs, no Rust Book, no
search engine, no LLM. The one tool you're explicitly allowed (encouraged,
even) to use is the compiler itself: reading a `rustc` error and reacting to
it correctly, fast, is exactly the skill this course exercises every day.

- **Finish all four fixes within 30 minutes, unaided** → you meet the bar
  this course assumes. Go on to the Environment Setup section of
  `../../README.md`.
- **Past 30 minutes and still stuck on two or more of the four** → this
  course's pace (five days, ~1 module per half-day, no time built in for
  ownership/borrowing remediation) is likely to leave you behind starting
  Day 1. Consider a prerequisite refresher first (Rust Book ch. 4
  "Understanding Ownership", ch. 10 "Generic Types, Traits, and Lifetimes",
  and the Rustonomicon's "Ownership" section for the parts the Book
  simplifies), or talk to the instructor about fit before day 1.
- **One exercise past 30 minutes, the other three clean** → a fine result.
  Mention which one to the instructor; it may just shift a few minutes of
  Day 1 pacing, not a signal you're in the wrong room.

This is a calibration tool, not a graded test, and nobody but you sees your
time. The only way to get a wrong reading is to look things up — that
measures whether you can search, not whether the reflex is already there.

## Running the check

```bash
source "$HOME/.cargo/env"
export CARGO_TARGET_DIR="/path/to/labs/target/prereq"   # or your own scratch dir
cd labs
cargo build -p prereq-check --features challenge
```

This is *expected to fail*. Behind the `challenge` feature are four small,
independently broken items, each broken for one distinct, ordinary
"I've-shipped-Rust-for-a-while" reason — not obscure trivia. Your job is to
make each one compile (and keep its behavior — see the tests) with the
smallest change that's actually correct, not the smallest change that makes
the error go away.

Once you believe you're done:

```bash
cargo test -p prereq-check --features challenge
```

All 4 tests should pass: `test_first_line`, `test_with_prefix`,
`test_banned_word_validator`, `test_get_or_insert`.

## The four exercises (in `src/lib.rs`)

1. **`first_line`** — a function returning a reference tied to the wrong
   lifetime.
2. **`WithPrefix`** — an iterator adapter struct that needs an explicit
   lifetime parameter (elision rules that apply to function signatures
   don't apply the same way to struct fields).
3. **`banned_word_validator`** — a trait-object return type that needs an
   explicit lifetime bound instead of the implicit `'static` default.
4. **`get_or_insert`** — a borrow-across-mutation conflict. The fix here is
   *not* "add another lifetime somewhere" — it's a smaller, different
   approach to the whole function.

If you blow past 30 minutes on one of these, the fixes (with the reasoning
attendee working through this yourself, don't open it before the clock
runs out; the exercise only calibrates correctly if you don't.

## Default build stays green

Without the `challenge` feature — the crate's default — `prereq-check` is
an intentional no-op:

```bash
cargo build -p prereq-check        # or: cargo build --workspace
```

This must always succeed. The self-check never blocks anyone else's build
of the other 9 labs, whether or not they've gotten to the prerequisite
check yet.

## Why a compile-fix exercise, not a quiz

A multiple-choice quiz on ownership rules tests recognition; production
Rust in this course's labs (raw pointers in Lab 2, `Pin`/`Waker` in Lab 7,
typestate + GATs in Lab 4) tests *production* — writing code the borrow
checker accepts on the first or second try, and knowing why it didn't on
the first. Fixing four small, real compile errors under a clock is a much
closer proxy for that than any quiz would be.
