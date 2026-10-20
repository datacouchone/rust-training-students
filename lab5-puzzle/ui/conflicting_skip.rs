// EXPECT: skip cannot be combined
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(skip, redact)]
    value: u8,
}
fn main() {}
