// EXPECT: WireSummary requires a struct with named fields
#[derive(lab5_puzzle::WireSummary)]
enum Event {
    Ping,
}
fn main() {}
