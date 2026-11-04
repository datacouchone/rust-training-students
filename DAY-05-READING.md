# Day 5 — slide reading points

The numbering matches the participant and coach decks. Run commands from the participant repository root. Line numbers below describe the reviewed starter; the coach highlights current saved lines.

## 1. Incidents and release decisions

Yesterday we measured and repaired performance. Today we ask what remains correct when work is interrupted, repeated or rolled back.

**ELI5:** A fast delivery service still needs to know whether a parcel was delivered before accepting a retry.

**Terms:** Incident = an observed violation of expected behavior; invariant = a rule that must remain true; evidence = a recorded observation with a reproducible trigger.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** The deliverable is a repair plus a defensible handoff. A successful first request is only a starting observation.

**Next:** Begin with the deployment problem: why tests alone cannot predict every real input or version combination.

## 2. Amazon: a tested release can still harm customers

Connect yesterday’s passing tests to the wider question of releasing a changed service.

**ELI5:** Testing one bridge crossing does not establish that every vehicle and weather condition is covered.

**Terms:** Canary = a limited exposure of a candidate version; compatibility = old and new versions interpret shared data and protocols consistently.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Use the public problem diagram. Separate the published deployment practice from our original synthetic gateway exercise; do not present the exercise as Amazon code.

**Next:** Next, identify the controls that make gradual exposure and rollback useful.

## 3. Amazon’s approach: staged rollout and automatic rollback

A small rollout limits exposure only if we can detect its failures and return safely.

**ELI5:** A return ticket is useful only if the old station can still accept your luggage.

**Terms:** Rollback = returning to a previous version; observation window = the period of evidence used for a decision; guardrail = a signal that stops progression.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Compare new-version signals separately from fleet averages. Explain one compatibility question before calling rollback safe; our receipt ledger disappears on process restart.

**Next:** Move from deployment context to the investigation sequence we will actually practise.

## 4. M9 and M10: diagnosis followed by release judgement

Turn the industry example into a repeatable classroom method.

**ELI5:** A mechanic observes the symptom, tests competing causes, then changes the part supported by evidence.

**Terms:** Observation = what happened; hypothesis = an explanation to test; regression = an automated check for a known failure.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Keep diagnosis, code correctness and operational release decisions separate. Name one result that belongs in each category.

**Next:** Show exactly which labs and exercises will practise that method.

## 5. Day 5 lab map: incident evidence to release decision

The class needs a map of demonstrations, individual practice and paired work.

**ELI5:** A driving lesson has an instructor demonstration, a supervised attempt and an independent journey.

**Terms:** Demo = an isolated teaching file; Lab 09 = framing plus accounting repair; Project Relay = the paired integration incident.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Nine independent demos support two Lab 09 repair blocks. The 150-minute hackathon is self-contained; participants do not need to copy earlier completed code into it.

**Next:** Agree on the schedule and the evidence each pair will hand over.

## 6. Today’s working sessions

Protect practice and review time instead of filling the whole day with explanation.

**ELI5:** Reserve time to inspect the repair, not only to write it.

**Terms:** Driver = person operating the editor; reviewer = person checking the contract and evidence; handoff = explanation another engineer can act on.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** The displayed schedule is a planning guide. Preserve the 150-minute paired block and allow a separate presentation round for a large cohort.

**Next:** Start the incident with observations before opening its implementation.

## 7. The gateway incident

Use the investigation method on three distinct symptoms.

**ELI5:** Three warning lights can have three different causes.

**Terms:** Probe = a small executable experiment; acceptance test = a check that asserts the required contract.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `production-labs/case09-incident/src/bin/probe.rs:9`

| Line | Code | Meaning |
|---|---|---|
| 10 | `let bytes = frame(b"12:100");` | Encode a fixed body with its four-byte length header; keep the same input before and after replacement. |
| 11 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 13 | `tx.write_all(&bytes[..6]).await.unwrap();` | Supply four header bytes plus two body bytes before timing out. |
| 16 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 19 | `d.buffered()` | Report progress stored in the decoder fields. Zero does not mean the underlying stream consumed no bytes. |
| 22 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 23 | `println!("event=retry result={:?}", d.next(&mut rx).await);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `production-labs/case09-incident/src/bin/probe.rs:25`

| Line | Code | Meaning |
|---|---|---|
| 28 | `println!("event=before accounts={:?}", g.accounts());` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 29 | `println!("event=request result={:?}", g.apply(b"1:1"));` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 30 | `println!("event=after accounts={:?}", g.accounts());` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `production-labs/case09-incident/src/bin/probe.rs:32`

| Line | Code | Meaning |
|---|---|---|
| 35 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 40 | `println!("event=footprint tenants={}", g.accounts().len());` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `production-labs/case09-incident/src/lib.rs:30`

| Line | Code | Meaning |
|---|---|---|
| 30 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 34 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 35 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 37 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 40 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 41 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 44 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 45 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `production-labs/case09-incident/src/lib.rs:77`

| Line | Code | Meaning |
|---|---|---|
| 77 | `pub fn apply(&mut self, body: &[u8]) -> Result<Account, Reject> {` | This synchronous method has exclusive access to its receiver through &mut self. Inspect whether all fallible work happens before it mutates stored state. |
| 78 | `let text = std::str::from_utf8(body).map_err(&#124;_&#124; Reject::Encoding)?;` | Validate text encoding before parsing or creating tenant state. |
| 79 | `let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;` | Require the tenant:byte_count separator before interpreting the two fields. |
| 80 | `let tenant = tenant.parse::<u32>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the tenant identity and return Number on invalid input. |
| 81 | `let bytes = bytes.parse::<u64>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the byte count without silently wrapping an oversized number. |
| 82 | `let account = self.accounts.entry(tenant).or_default();` | This starter entry/or_default can create an account immediately, before later arithmetic succeeds. |
| 83 | `account.accepted = account.accepted.checked_add(1).ok_or(Reject::Overflow)?;` | BEFORE: this commits the count before checking the byte total; a later error leaves partial state. |
| 84 | `account.bytes = account.bytes.checked_add(bytes).ok_or(Reject::Overflow)?;` | BEFORE: checked arithmetic protects this one addition but cannot undo a previous field update. |

**Read the evidence:** Run the three listed probes. Record timeout/decode behavior, state after rejection and tenant growth separately. Normal process exit can still report faulty behavior.

**Next:** First isolate the framing rule for a byte stream.

## 8. A byte stream has no message boundaries

Before cancellation, establish what a complete message means.

**ELI5:** A conveyor belt carries bytes; the receiver needs labels saying where each parcel ends.

**Terms:** Frame = length prefix plus body; big endian = most significant length byte first; EOF = no more bytes from the producer.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/stream_boundaries.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/stream_boundaries.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 12 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 16 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 18 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 24 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Both versions print two bodies and clean EOF. Bytes [49,58,49,48] are the text 1:10; [50,58,50,48] are 2:20. Same output is expected: this input does not exercise cancellation.

**Next:** Now interrupt the header after only two bytes arrive.

## 9. Cancellation can discard local progress

Complete frames passed, so change the failure schedule while keeping the same protocol.

**ELI5:** If you throw away your note after reading half an address, the unread envelope cannot restore the first half.

**Terms:** Future = suspended computation and its state; cancellation here = dropping the timed-out next future; decoder = owner that survives the next call.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/cancel_header.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/cancel_header.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 12 | `let bytes = frame(b"1:100");` | Encode a fixed body with its four-byte length header; keep the same input before and after replacement. |
| 13 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 15 | `tx.write_all(&bytes[..2]).await.unwrap();` | Supply only two header bytes before timing out the read. |
| 16 | `let result = timeout(Duration::from_millis(1), d.next(&mut rx)).await;` | Bound the read attempt; a timeout drops that attempt’s future, while the Decoder variable remains available. |
| 17 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 20 | `d.buffered()` | Report progress stored in the decoder fields. Zero does not mean the underlying stream consumed no bytes. |
| 23 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 24 | `println!("resumed={:?}", d.next(&mut rx).await);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Before: timeout=true, retained decoder progress=0, resume reports frame limit. After: retained progress=2 and body bytes [49,58,49,48,48] reconstruct 1:100. Zero buffered does not mean zero bytes were consumed from the stream.

**Next:** Retaining the header alone is insufficient; interrupt the body next.

## 10. Persist progress in the decoder

The same cancellation problem can occur after a complete header and a partial body.

**ELI5:** Keep both the parcel label and how much of its contents has arrived.

**Terms:** header_read = saved header byte count; body_read = saved body byte count; length = validated body size.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/cancel_body.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/cancel_body.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 12 | `let bytes = frame(b"1:100");` | Encode a fixed body with its four-byte length header; keep the same input before and after replacement. |
| 13 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 15 | `tx.write_all(&bytes[..6]).await.unwrap();` | Supply four header bytes plus two body bytes before timing out. |
| 16 | `let result = timeout(Duration::from_millis(1), d.next(&mut rx)).await;` | Bound the read attempt; a timeout drops that attempt’s future, while the Decoder variable remains available. |
| 17 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 20 | `d.buffered()` | Report progress stored in the decoder fields. Zero does not mean the underlying stream consumed no bytes. |
| 23 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 24 | `println!("resumed={:?}", d.next(&mut rx).await);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Before: retained_header_and_body=0 and resume returns None. After: retained progress=6, consisting of four header and two body bytes, and the original body is reconstructed. The driver uses an in-memory duplex stream and a paused Tokio clock.

**Next:** Distinguish legitimate end-of-stream from an incomplete frame.

## 11. EOF, empty frame and truncation

A reader must preserve the meaning of its result as well as its bytes.

**ELI5:** An empty parcel is different from no parcel, and both differ from a parcel cut in half.

**Terms:** Option = frame present or clean EOF; Result = success or error; Some(empty Vec) = a valid empty payload.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/truncated_eof.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/truncated_eof.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 13 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 15 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 16 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** No bytes yields Ok(None); a zero-length header yields Ok(Some([])); a partial header must yield UnexpectedEof. Only the third observation changes after the repair.

**Next:** Next, decide what to do after an impossible announced length.

## 12. Length limits and terminal stream errors

An invalid frame creates both a resource problem and a question about stream reuse.

**ELI5:** After a broken parcel label, do not guess where the next parcel starts.

**Terms:** Frame limit = maximum allowed body length; poisoned state = terminal decoder error that forbids pretending to resume normally.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/poisoned_stream.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/poisoned_stream.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 12 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 15 | `println!("oversize={:?}", d.next(&mut rx).await);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 16 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 20 | `println!("An invalid stream must fail immediately on reuse, not wait for a new header.");` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Both versions reject the oversized length. Before, retry waits until the outer timeout returns Err(Elapsed). After, the outer timeout succeeds with an inner InvalidData error: Ok(Err(...stream poisoned...)). Explain both result layers.

**Next:** Test many interruption points instead of trusting one example.

## 13. Regression at every byte boundary

The previous demos establish mechanisms; now exercise every byte split in one frame.

**ELI5:** Check every possible cut in the same ribbon and verify that joining it still reconstructs the original.

**Terms:** Boundary sweep = repeating an experiment at every split position; differential check = compare reconstructed bytes with the expected bytes.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/every_boundary.rs:31`

| Line | Code | Meaning |
|---|---|---|
| 31 | `pub async fn next<R: AsyncRead + Unpin>(` | This async method borrows the decoder and reader; the returned future can suspend while those owners remain alive. |
| 35 | `let mut header = [0u8; 4];` | BEFORE: these header bytes belong to this call’s future. Cancelling it loses the local progress record. |
| 36 | `match reader.read_exact(&mut header).await {` | Read all four header bytes, possibly consuming a prefix before the future is dropped. A retry cannot assume those bytes remain in the stream. |
| 38 | `Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),` | BEFORE: this conflates a partial header with clean EOF. The repaired reader distinguishes their saved byte counts. |
| 41 | `let n = u32::from_be_bytes(header) as usize;` | Interpret the four protocol bytes as one big-endian unsigned length, then use it as a host index. |
| 42 | `if n > MAX_FRAME {` | Reject the announced length before allocating or resizing a body buffer. |
| 45 | `let mut body = vec![0; n];` | BEFORE: the body buffer is local too; partially read bytes disappear with a cancelled future. |
| 46 | `reader.read_exact(&mut body).await?;` | Fill the body; this all-or-nothing-looking API does not make a cancelled read transactional. |

**Code:** `day05-demos/src/bin/every_boundary.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 12 | `let bytes = frame(b"1:100");` | Encode a fixed body with its four-byte length header; keep the same input before and after replacement. |
| 14 | `for split in 1..bytes.len() {` | Try each interior byte boundary separately with a fresh stream and decoder. |
| 15 | `let (mut tx, mut rx) = duplex(64);` | Create a bounded in-memory asynchronous byte stream for a deterministic I/O experiment; this is not a TCP latency measurement. |
| 20 | `drop(tx);` | Close the producer so the reader can distinguish complete input from EOF or truncation. |
| 23 | `println!("split={split} preserved={ok}");` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 25 | `println!("preserved={preserved}/{}", bytes.len() - 1);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** The nine-byte frame has eight interior split positions. Before reports preserved=0/8; after reports preserved=8/8. This is evidence for those inputs and schedules, not a proof for every possible I/O source.

**Next:** Hand the independent framing task to participants.

## 14. Lab 09 — framing repair

Move from isolated demonstrations to the participant’s own Decoder implementation.

**ELI5:** The instructor has shown the repair method; now participants apply it to their own object.

**Terms:** Persistent state = fields retained between calls; resume point = the unfilled suffix and its recorded count.

Attempt the lab, save your work, run the focused checks, and explain the evidence.

**Read / check:** `production-labs/case09-incident/src/lib.rs:30`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:5`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:23`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:34`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:53`

**Read the evidence:** Baseline tests exercise supplied infrastructure. The focused cancellation regression should select exactly one test. Whole-lab tests can still fail on accounting. Discuss the worked review after your attempt.

**Next:** After framing practice, inspect an error that leaves accounting partially changed.

## 15. An error can still leave corrupted state

Cancellation is not the only way to break a state contract: an ordinary error return can do it.

**ELI5:** A cashier increments the sale count before discovering that the payment cannot be recorded.

**Terms:** Overflow = a value cannot fit its integer type; transactional rejection = an error leaves the promised state unchanged.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/overflow_update.rs:78`

| Line | Code | Meaning |
|---|---|---|
| 78 | `pub fn apply(&mut self, body: &[u8]) -> Result<Account, Reject> {` | This synchronous method has exclusive access to its receiver through &mut self. Inspect whether all fallible work happens before it mutates stored state. |
| 79 | `let text = std::str::from_utf8(body).map_err(&#124;_&#124; Reject::Encoding)?;` | Validate text encoding before parsing or creating tenant state. |
| 80 | `let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;` | Require the tenant:byte_count separator before interpreting the two fields. |
| 81 | `let tenant = tenant.parse::<u32>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the tenant identity and return Number on invalid input. |
| 82 | `let bytes = bytes.parse::<u64>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the byte count without silently wrapping an oversized number. |
| 83 | `let account = self.accounts.entry(tenant).or_default();` | This starter entry/or_default can create an account immediately, before later arithmetic succeeds. |
| 84 | `account.accepted = account.accepted.checked_add(1).ok_or(Reject::Overflow)?;` | BEFORE: this commits the count before checking the byte total; a later error leaves partial state. |
| 85 | `account.bytes = account.bytes.checked_add(bytes).ok_or(Reject::Overflow)?;` | BEFORE: checked arithmetic protects this one addition but cannot undo a previous field update. |

**Code:** `day05-demos/src/bin/overflow_update.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 14 | `let before = g.accounts().clone();` | Keep a snapshot so the example can compare stored state after a rejected request. |
| 16 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Both versions return Err(Overflow). Before, accepted changes from 1 to 2 while bytes remain u64::MAX. After, accepted remains 1. The error value alone cannot show whether rejection was safe.

**Next:** Make the prepare and commit boundary visible in the next example.

## 16. Prepare, then commit

Retain the same rejection but move all fallible calculations before mutation.

**ELI5:** Write the proposed balance on scratch paper, check it, then replace the official balance once.

**Terms:** Candidate state = a locally computed proposal; commit = the step that publishes it into the account map.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/prepare_commit.rs:78`

| Line | Code | Meaning |
|---|---|---|
| 78 | `pub fn apply(&mut self, body: &[u8]) -> Result<Account, Reject> {` | This synchronous method has exclusive access to its receiver through &mut self. Inspect whether all fallible work happens before it mutates stored state. |
| 79 | `let text = std::str::from_utf8(body).map_err(&#124;_&#124; Reject::Encoding)?;` | Validate text encoding before parsing or creating tenant state. |
| 80 | `let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;` | Require the tenant:byte_count separator before interpreting the two fields. |
| 81 | `let tenant = tenant.parse::<u32>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the tenant identity and return Number on invalid input. |
| 82 | `let bytes = bytes.parse::<u64>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the byte count without silently wrapping an oversized number. |
| 83 | `let account = self.accounts.entry(tenant).or_default();` | This starter entry/or_default can create an account immediately, before later arithmetic succeeds. |
| 84 | `account.accepted = account.accepted.checked_add(1).ok_or(Reject::Overflow)?;` | BEFORE: this commits the count before checking the byte total; a later error leaves partial state. |
| 85 | `account.bytes = account.bytes.checked_add(bytes).ok_or(Reject::Overflow)?;` | BEFORE: checked arithmetic protects this one addition but cannot undo a previous field update. |

**Code:** `day05-demos/src/bin/prepare_commit.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 14 | `let before = g.accounts().clone();` | Keep a snapshot so the example can compare stored state after a rejected request. |
| 16 | `println!("reject={result:?} unchanged={}", g.accounts() == &before);` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 17 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Before prints unchanged=false; after prints unchanged=true. There is no await in apply, and &mut self provides exclusive access through this API. This argument is in-memory; it does not provide crash durability.

**Next:** Add a separate limit for how many tenant identities may exist.

## 17. Cardinality is a resource boundary

Correct arithmetic still permits unbounded growth in the number of keys.

**ELI5:** Small parcels still fill a warehouse if there is no limit on how many shelves are created.

**Terms:** Cardinality = number of distinct identities; existing-key update = change to a record already counted against the limit.

Predict → run original → inspect the instructor change → rerun the same input → compare.

**Code:** `day05-demos/src/scenarios/tenant_limit.rs:78`

| Line | Code | Meaning |
|---|---|---|
| 78 | `pub fn apply(&mut self, body: &[u8]) -> Result<Account, Reject> {` | This synchronous method has exclusive access to its receiver through &mut self. Inspect whether all fallible work happens before it mutates stored state. |
| 79 | `let text = std::str::from_utf8(body).map_err(&#124;_&#124; Reject::Encoding)?;` | Validate text encoding before parsing or creating tenant state. |
| 80 | `let (tenant, bytes) = text.split_once(':').ok_or(Reject::Shape)?;` | Require the tenant:byte_count separator before interpreting the two fields. |
| 81 | `let tenant = tenant.parse::<u32>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the tenant identity and return Number on invalid input. |
| 82 | `let bytes = bytes.parse::<u64>().map_err(&#124;_&#124; Reject::Number)?;` | Parse the byte count without silently wrapping an oversized number. |
| 83 | `let account = self.accounts.entry(tenant).or_default();` | This starter entry/or_default can create an account immediately, before later arithmetic succeeds. |
| 84 | `account.accepted = account.accepted.checked_add(1).ok_or(Reject::Overflow)?;` | BEFORE: this commits the count before checking the byte total; a later error leaves partial state. |
| 85 | `account.bytes = account.bytes.checked_add(bytes).ok_or(Reject::Overflow)?;` | BEFORE: checked arithmetic protects this one addition but cannot undo a previous field update. |

**Code:** `day05-demos/src/bin/tenant_limit.rs:11`

| Line | Code | Meaning |
|---|---|---|
| 11 | `async fn main() {` | This is the runnable input driver. It calls the implementation shown in the other file and prints the resulting observations. |
| 14 | `println!("new_tenant={:?}", g.apply(b"2:20"));` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 15 | `println!("existing_tenant={:?}", g.apply(b"1:5"));` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 16 | `println!("accounts={:?}", g.accounts());` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** With capacity one, before accepts tenant 2 and leaves two accounts. After rejects tenant 2 with Cardinality, still allows tenant 1 to reach accepted=2 and bytes=15, and keeps one account.

**Next:** Let participants combine parsing, arithmetic and cardinality checks in apply.

## 18. Lab 09 — accounting repair

The accounting exercise combines the separate rejection mechanisms.

**ELI5:** Check every condition before making the first entry in the official ledger.

**Terms:** Parsing = interpreting bytes as values; map entry creation = a mutation even before its fields are changed.

Attempt the lab, save your work, run the focused checks, and explain the evidence.

**Read / check:** `production-labs/case09-incident/src/lib.rs:77`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:66`

**Read / check:** `production-labs/case09-incident/tests/acceptance.rs:79`

**Read the evidence:** Run the named invalid-frame test and check that one test is selected. Also use the overflow and cardinality acceptance cases. Framing and accounting are separate tasks; explain any remaining whole-lab failures by their actual names.

**Next:** Now discuss which runtime evidence helps explain an incident outside a tiny regression.

## 19. Structured events need correlation

A reproducible local test is clear; interleaved production events need identity.

**ELI5:** Label each conversation before trying to reconstruct who answered whom.

**Terms:** Span = context for one operation; event = an observation inside that context; correlation ID = identity used to connect observations.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Code:** `labs/lab9-core-dump/starter/src/server.rs:89`

| Line | Code | Meaning |
|---|---|---|
| 89 | `#[tracing::instrument(skip(bytes, state))]` | Create operation context while excluding the named large or sensitive arguments from automatic field recording. |
| 90 | `async fn process_frame(conn_id: u64, frame_no: u64, bytes: &[u8], state: &AppState) -> String {` | AFTER: prepare the proposed byte total; failure here must leave the stored account untouched. |
| 102 | `let metric = parsed.unwrap();` | This injected fault panics on malformed input instead of returning an expected parse error. |

**Read the evidence:** Source review only. Show instrument(skip(bytes,state)) and the conn_id/frame_no arguments. These identifiers can be recorded while large or sensitive payload fields are excluded. Do not claim a live log capture.

**Next:** Use that context to separate a dependency cycle from ordinary slow progress.

## 20. Deadlock versus slow work

A timeout says work did not finish in time; it does not identify why.

**ELI5:** Two people each hold one key and wait for the key held by the other.

**Terms:** Deadlock = a wait cycle with no progress under the ownership arrangement; serialization = work proceeds one at a time.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Code:** `labs/lab9-core-dump/starter/src/server.rs:149`

| Line | Code | Meaning |
|---|---|---|
| 152 | `tokio::spawn(` | Create separately scheduled work. Ownership of the handle and cancellation behavior require an explicit argument. |
| 154 | `let _guard_a = a1.lock().await;` | Inspect which lock is already held when this path asks for another; compare the opposite order on the other path. |
| 156 | `tokio::time::sleep(Duration::from_millis(100)).await;` | A long wait can keep a detached task alive after its initiating request has completed. |
| 158 | `let _guard_b = b1.lock().await; // never returns under this fault` | Compare the lock dependency with the other task; a cycle explains deadlock, whereas one long-held lock may only serialize. |
| 166 | `tokio::spawn(` | Create separately scheduled work. Ownership of the handle and cancellation behavior require an explicit argument. |
| 168 | `let _guard_b = b2.lock().await;` | Task B acquires the registry lock first, reversing task A’s metrics-first order. |
| 170 | `tokio::time::sleep(Duration::from_millis(100)).await;` | A long wait can keep a detached task alive after its initiating request has completed. |
| 172 | `let _guard_a = a2.lock().await; // never returns under this fault` | Task B still holds registry while waiting for metrics; task A holds metrics while waiting for registry, creating a wait cycle. |

**Read the evidence:** Trace A then B on one path and B then A on the other. Explain the wait cycle. A lock held across enrichment can serialize requests without creating that cycle; choose the mechanism supported by evidence.

**Next:** Next ask whether work remains alive after its request is gone.

## 21. Task leakage after each request

Even a completed response can leave background work without a request owner.

**ELI5:** Closing the order screen does not automatically stop a worker you hired separately.

**Terms:** JoinHandle = handle to a spawned task; detach = allow the task to continue after its handle is dropped; abort = request cancellation.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Code:** `labs/lab9-core-dump/starter/src/server.rs:128`

| Line | Code | Meaning |
|---|---|---|
| 129 | `tokio::spawn(` | Create separately scheduled work. Ownership of the handle and cancellation behavior require an explicit argument. |
| 133 | `tokio::time::sleep(Duration::from_secs(3600)).await;` | A long wait can keep a detached task alive after its initiating request has completed. |

**Read the evidence:** Source review only. The spawned task waits for a very long time after the handler returns. Dropping its JoinHandle does not cancel it. Connect task counters to a guard whose lifetime actually covers the work.

**Next:** Distinguish a task failure from a process failure.

## 22. Panic scope and process scope

A failed operation can affect different ownership boundaries.

**ELI5:** One worker stopping is different from shutting down the whole building.

**Terms:** unwrap = extract success or panic; JoinError = failure reported when joining a task; panic=abort = process-termination strategy.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Code:** `labs/lab9-core-dump/starter/src/server.rs:89`

| Line | Code | Meaning |
|---|---|---|
| 89 | `#[tracing::instrument(skip(bytes, state))]` | Create operation context while excluding the named large or sensitive arguments from automatic field recording. |
| 90 | `async fn process_frame(conn_id: u64, frame_no: u64, bytes: &[u8], state: &AppState) -> String {` | AFTER: prepare the proposed byte total; failure here must leave the stored account untouched. |
| 102 | `let metric = parsed.unwrap();` | This injected fault panics on malformed input instead of returning an expected parse error. |

**Read the evidence:** Point to parsed.unwrap() in the injected fault branch. Expected malformed input should be handled as an error. The observed boundary depends on task handling and panic strategy; no process crash is demonstrated on this slide.

**Next:** Choose evidence that can show suspended tasks as well as running threads.

## 23. Live demo: three async failures

We have read three faults. Now watch each one fail over real localhost TCP, then run its repaired implementation against the same trigger.

```sh
python3 day05-incident-demo/run.py all
python3 day05-incident-demo/run.py deadlock
python3 day05-incident-demo/run.py leak
python3 day05-incident-demo/run.py panic
```

Deadlock: TIMEOUT becomes OK. Leak: live tasks grow 1, 2, 3 before and remain zero after. Panic: malformed input gives EOF and JoinError before; ERR after. A second valid connection succeeds. Cleanup returns tracked tasks to zero.

- BEFORE · opposing lock order: `day05-incident-demo/src/main.rs:57` — Both tasks acquire their first lock, rendezvous at the barrier, then request the other. The barrier establishes the cycle without relying on sleep timing.
- AFTER · consistent lock order: `day05-incident-demo/src/main.rs:89` — Both tasks acquire metrics before registry. The second task waits without holding the resource the first needs.
- BEFORE · task survives its response: `day05-incident-demo/src/main.rs:109` — Each frame starts an infinite sleeper. A lifetime guard counts the real future. The demo supervisor retains a handle solely to clean up after observation; the original lab discards its handle.
- AFTER · request joins finite work: `day05-incident-demo/src/main.rs:122` — The child performs a finite job and the request awaits its handle before replying. Storing a handle alone would not be a repair.
- BEFORE · unwrap external input: `day05-incident-demo/src/main.rs:132` — Invalid numeric input panics in the connection task.
- AFTER · return a normal rejection: `day05-incident-demo/src/main.rs:137` — Match the parse Result and return ERR on malformed input.
- TCP handler and correlation: `day05-incident-demo/src/main.rs:161` — This is the request handler used by every comparison. The span records connection identity, frame number, fault and repaired mode; payload and shared state are skipped.
- The implementation boundary to explain: `labs/lab9-core-dump/starter/src/tracing_init.rs:19` — Follow the named function’s inputs, retained state and return or suspension points. A CPU profile is suited to where CPU time is spent. A task view is suited to logical async lifetimes. Choose the tool from the hypothesis instead of opening every tool and hoping one names the bug.

## 24. Capture native stacks and async evidence

Capture native evidence from a dedicated child with the same lock cycle: Linux uses GDB all-thread backtraces; macOS uses sample. Compare the lock events with worker stacks. Linux participants follow day05-incident-demo/LINUX-EXERCISE.md.

```sh
python3 day05-incident-demo/run.py evidence
```

SAMPLE_READY confirms the lock-cycle setup. Read native-sample.txt and sample-child.log together. Linux GDB stops at classroom_capture_ready: #0/#1 are stack frames, not sample counts. Parked workers alone do not prove deadlock.

- Dedicated process being sampled: `day05-incident-demo/src/main.rs:413` — This child arms the same deterministic lock cycle and waits. The capture script samples only this PID, then terminates that child.

## 25. Preserve the incident evidence

Open the actual evidence bundle and its incident report. Preserve the source identity, exact binary identity when sampled, raw logs, timestamp and native snapshot.

```sh
python3 day05-incident-demo/run.py report
python3 day05-incident-demo/run.py evidence
```

The report is generated from recorded executions. This is a Linux GDB backtrace snapshot or macOS live sample, not a Linux core dump. The existing Linux/GDB guide remains a separate platform exercise.

- What the captured process was doing: `day05-incident-demo/src/main.rs:413` — Tie the sample to the exact task setup. A native sample is a different artifact from a core dump.

## 26. Run regressions; explain their limits

Run the executable regressions for the faults we just observed. Each check has a concrete trigger and checks the repaired behavior.

```sh
python3 day05-incident-demo/run.py test
```

Five tests must pass: lock-cycle repair, task lifetime, malformed input, replay identity and release decisions. Local regressions do not establish all schedules, production latency or release safety.

- The before/after assertions: `day05-incident-demo/src/main.rs:284` — The checks assert timeout and two waiting tasks before the lock repair, reply equivalence and zero retained tasks after the leak repair, and ERR rather than a panic after parser repair.

## 27. Lost acknowledgement: retry the same request

The server commits a request over TCP, but the client discards its acknowledgement. Retry the same tenant/request identity and then send conflicting content.

```sh
python3 day05-incident-demo/run.py retry
```

BEFORE charges three times. AFTER charges once: identical retry returns REPLAY; changed content under the same identity returns CONFLICT. This ledger is in memory for this process lifetime.

- TCP commit, lost ACK and replay decision: `day05-incident-demo/src/main.rs:309` — The first response is written but deliberately not read by the client. A receipt maps tenant/request identity to content; identical retry reuses it, while conflicting content rejects.

## 28. Review the repairs against evidence

Rerun the three faults and generate a comparison table from their actual output. Challenge each proposed shortcut against its failure mechanism.

```sh
python3 day05-incident-demo/run.py review
```

A longer timeout does not break the lock cycle. Retaining a task handle alone does not stop the task. Catching a panic does not validate external input. Defend each repair using the raw observed counts and outcomes.

- Which observations justify each repair: `day05-incident-demo/src/main.rs:284` — This comparison asserts the fault triggers and repaired outcomes, then the review command extracts the actual observations into a table.

## 29. Run the canary decision rules

Execute the release decision function over five explicitly labelled teaching windows. Predict the decision before reading each result.

```sh
python3 day05-incident-demo/run.py rollout
```

Expected decisions: HOLD, HOLD, ROLLBACK, ROLLBACK, PROMOTE. These are fixture inputs and exercise thresholds, not measured production traffic. Empty evidence, small samples, duplicates and latency have different implications.

- Release decision implementation: `day05-incident-demo/src/main.rs:363` — Validate the evidence first; duplicate effects cause rollback; too few observations cause hold; then evaluate errors and latency.
- Five input windows: `day05-incident-demo/src/main.rs:376` — These visible fixture values are passed into the actual decision function. The result is calculated, not a hand-written slide label.

## 30. Generate the incident handoff

Generate the handoff from the runs saved on this machine. Show the captured files and separate observed results from remaining uncertainties.

```sh
python3 day05-incident-demo/run.py report
```

The report lists the actual run timestamps and source hashes, links raw evidence and states platform and durability limits. No core capture or live tokio-console session is claimed.

- Assertions behind the handoff: `day05-incident-demo/src/main.rs:284` — The handoff links the run output generated by these checks. A report without its raw evidence is insufficient.

## 31. Project Relay: your pair owns the incident

Project Relay combines the mechanisms in one self-contained ingestion service.

**ELI5:** Two engineers take ownership of a service, not merely a collection of isolated edits.

**Terms:** Core repair track = required contracts; stretch work = optional further investigation; baseline = smoke test before repairs.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `day05-hackathon/src/worker.rs:53`

| Line | Code | Meaning |
|---|---|---|
| 53 | `pub async fn submit(` | Trace admission, spawned enrichment, deadline and commit ownership through the public request entry point. |
| 64 | `let child = tokio::spawn(Arc::clone(self).operation(e, delay));` | Create separately scheduled work. Ownership of the handle and cancellation behavior require an explicit argument. |
| 65 | `let result = timeout_at(deadline, child)` | Bound the read attempt; a timeout drops that attempt’s future, while the Decoder variable remains available. |

**Code:** `day05-hackathon/src/main.rs:149`

| Line | Code | Meaning |
|---|---|---|
| 156 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `day05-hackathon/src/main.rs:173`

| Line | Code | Meaning |
|---|---|---|
| 192 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 198 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 204 | `tokio::time::sleep(Duration::from_millis(if slow {` | A long wait can keep a detached task alive after its initiating request has completed. |
| 210 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 215 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `day05-hackathon/src/fixture.rs:25`

| Line | Code | Meaning |
|---|---|---|
| 32 | `let task = tokio::spawn(async move {` | Create separately scheduled work. Ownership of the handle and cancellation behavior require an explicit argument. |

**Read the evidence:** Show the guide first, then run the load probe. It uses actual loopback TCP; deterministic cancellation regressions use duplex I/O. Initial acceptance failures are expected. Preserve fixtures, tests and limits.

**Next:** Trace the same request through the five days of concepts.

## 32. Five days meet in one request

Connect each repair station to the earlier concept it exercises.

**ELI5:** A parcel crosses several checkpoints; every checkpoint must preserve the same identity and ownership.

**Terms:** ABI layout = target-specific native arrangement; wire format = explicit byte contract; validated type = input that passed the trust boundary.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `day05-hackathon/src/boundary.rs:7`

| Line | Code | Meaning |
|---|---|---|
| 7 | `pub fn bytes_equal(a: &[u8], b: &[u8]) -> bool {` | Review the borrowed-slice contract and the distinction between complete equality and common-prefix equality. |
| 9 | `let n = a.len().min(b.len());` | Comparing only the common prefix can call a shorter and longer byte string equal; memory safety alone does not establish semantic equality. |
| 14 | `unsafe { memcmp(a.as_ptr().cast(), b.as_ptr().cast(), n) == 0 }` | The C function reads only the supplied valid range and retains no pointer. Explain slice validity and length checks separately from equality semantics. |

**Code:** `day05-hackathon/src/main.rs:44`

| Line | Code | Meaning |
|---|---|---|
| 45 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 51 | `println!("Do not serialize struct padding. Native layout is platform-specific.");` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** The layout probe reports an observed native layout; it does not define the wire format. bytes_equal currently compares a common prefix. The supplied C call borrows valid bounded slices and retains no pointers; the bug is incomplete equality, not deliberately introduced undefined behavior.

**Next:** Assign driver and reviewer responsibilities before editing.

## 33. Two engineers, one handoff

A pair needs a shared contract before dividing implementation work.

**ELI5:** One person drives while the other reads the map and checks that the route reaches the intended destination.

**Terms:** Reviewer = active owner of counterexamples and evidence; role swap = transfer of implementation and explanation responsibility.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Start with fifteen minutes of evidence. Swap at minute fifty. At minute eighty-five compare measurements, then at one hundred fifteen prepare the release argument. Use one hint level at a time and record it.

**Next:** Introduce the lost-acknowledgement event card at its checkpoint.

## 34. Incident update: the acknowledgement was lost

The caller’s uncertainty must not create a second business operation.

**ELI5:** Ask for another receipt for the same purchase, not a second purchase with a new number.

**Terms:** Identity = tenant plus request ID; receipt = retained record of an accepted operation; conflict = same identity with a changed body.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `day05-hackathon/src/ledger.rs:34`

| Line | Code | Meaning |
|---|---|---|
| 34 | `pub fn apply(&mut self, e: &Validated) -> Result<Outcome, Reject> {` | This synchronous method has exclusive access to its receiver through &mut self. Inspect whether all fallible work happens before it mutates stored state. |
| 43 | `let a = self.accounts.entry(e.tenant()).or_default();` | This starter entry/or_default can create an account immediately, before later arithmetic succeeds. |

**Code:** `day05-hackathon/src/main.rs:160`

| Line | Code | Meaning |
|---|---|---|
| 163 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 166 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 170 | `println!("account={:?}", service.ledger.lock().await.accounts.get(&7));` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Run the replay probe and inspect both replies and ledger state. The fixture deliberately replays after treating the first acknowledgement as lost. At capacity, existing replay must remain recognizable while new identities reject; eviction changes the promise.

**Next:** Measure the repaired service while keeping successful outcomes visible.

## 35. Repair first; measure the same service

A faster response can hide rejection or unfinished background work.

**ELI5:** Finishing the queue by throwing away orders is not higher useful throughput.

**Terms:** Outcome count = applied/replayed/rejected/timed out; retained capacity = allocation kept after logical clearing; p95 = an observed percentile for this sample.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `day05-hackathon/src/analytics.rs:12`

| Line | Code | Meaning |
|---|---|---|
| 12 | `pub fn summarize(rows: &[Row], threshold: u32) -> Summary {` | Identify the complete selection rule and ensure count and total refer to the same valid rows. |
| 16 | `.filter(&#124;r&#124; r.status >= 500 && r.latency >= threshold)` | Inspect every predicate term before timing the query; omitted null or upper-status checks change the answer. |
| 17 | `.collect();` | Materializing selected values allocates intermediate storage; changing it requires preserving selection and aggregation semantics. |

**Code:** `day05-hackathon/src/main.rs:149`

| Line | Code | Meaning |
|---|---|---|
| 156 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `day05-hackathon/src/main.rs:173`

| Line | Code | Meaning |
|---|---|---|
| 192 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 198 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 204 | `tokio::time::sleep(Duration::from_millis(if slow {` | A long wait can keep a detached task alive after its initiating request has completed. |
| 210 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |
| 215 | `println!(` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Code:** `day05-hackathon/src/worker.rs:42`

| Line | Code | Meaning |
|---|---|---|
| 50 | `tokio::time::sleep(delay).await;` | A long wait can keep a detached task alive after its initiating request has completed. |

**Read the evidence:** Use identical client count and release settings. Compare outcomes, active work at response and after settling, capacity and exact query results. Small native batches do not establish a production SLO. Keep raw trials, including a losing case.

**Next:** Finish with a release decision both partners can defend.

## 36. Demo day: ship, hold or roll back?

Turn the pair’s repair and measurement into an engineering handoff.

**ELI5:** One engineer demonstrates the repair; the other must explain why it works and when to stop using it.

**Terms:** Gate = explicit decision function; rollback trigger = observed condition for stopping; compatibility argument = what remains readable and valid across versions.

Run the supplied example and interpret its result; no prepared replacement on this slide.

**Code:** `day05-hackathon/src/rollout.rs:17`

| Line | Code | Meaning |
|---|---|---|
| 17 | `pub fn decide(w: Window) -> Decision {` | This function translates evidence into a release decision. Compare every branch with the documented exercise policy. |
| 20 | `Decision::Promote` | Promotion needs sufficient consistent evidence; a default unconditional promotion does not implement the gate contract. |

**Code:** `day05-hackathon/src/main.rs:110`

| Line | Code | Meaning |
|---|---|---|
| 143 | `println!("window={name} decision={:?}", decide(w));` | Print an observation; a normal exit is not equivalent to the required behavior being correct. |

**Read the evidence:** Run the rollout probe and compare its output with the documented gate contract. The faulty starter can promote weak evidence. A completed gate must distinguish Hold, Rollback and Promote; the result authorizes no real deployment.

**Next:** Close with five short teach-back questions and record unresolved gaps.

## 37. Five-question teach-back

Review what participants can explain and demonstrate, rather than counting completed slides.

**ELI5:** Ask the other engineer to describe the route without handing them the narrator’s script.

**Terms:** Teach-back = participant explains the mechanism and its limit; demonstrated gap = something still unsupported by explanation or evidence.

Source walkthrough and discussion; no terminal run or replacement on this slide.

**Read the evidence:** Ask the five existing public questions one at a time. Use the private answer notes after each response. Record demonstrated results and unresolved issues; do not infer mastery from attendance or a green test alone.

**Next:** Finish with each pair’s evidence, release decision and next investigation.
