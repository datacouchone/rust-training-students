// EXPECT: E0382
use lab4_puzzle::Journal;
fn main() {
    let mut journal = Journal::<u32, 3>::new();
    let mut open = journal.begin();
    open.stage(1).unwrap();
    let prepared = open.prepare();
    open.stage(2).unwrap();
    drop(prepared);
}
