// EXPECT: Debug
struct NoDebug;
#[derive(lab5_puzzle::WireSummary)]
struct Event {
    value: NoDebug,
}
fn main() {
    let _ = Event { value: NoDebug }.wire_summary();
}
