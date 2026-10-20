// EXPECT: E0599
use lab4_puzzle::Journal;
fn main() {
    let mut journal = Journal::<u32, 3>::new();
    let mut open = journal.begin();
    open.stage(1).unwrap();
    open.commit();
}
