// EXPECT: E0502
use lab4_puzzle::Journal;
fn main() {
    let mut journal = Journal::<u32, 3>::new();
    let mut open = journal.begin();
    println!("{}", journal.len());
    open.stage(1).unwrap();
}
