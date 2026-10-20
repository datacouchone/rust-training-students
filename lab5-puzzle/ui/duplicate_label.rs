// EXPECT: duplicate output label
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(rename = "id")]
    first: u8,
    id: u8,
}
fn main() {}
