// EXPECT: unknown summary option
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    #[summary(omit)]
    value: u8,
}
fn main() {}
