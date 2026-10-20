// EXPECT: expected string literal
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(rename = 123)]
    value: u8,
}
fn main() {}
