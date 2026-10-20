// EXPECT: rename must not be empty
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(rename = "")]
    value: u8,
}
fn main() {}
