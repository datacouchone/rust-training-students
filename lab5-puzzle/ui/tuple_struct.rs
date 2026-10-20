// EXPECT: WireSummary requires a struct with named fields
#[derive(lab5_puzzle::WireSummary)]
struct Event(u8);
fn main() {}
