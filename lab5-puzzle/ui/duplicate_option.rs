// EXPECT: duplicate summary option
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(rename = "a")]
    #[summary(rename = "b")]
    value: u8,
}
fn main() {}
