// EXPECT: E0499
use lab4_puzzle::{Journal, LendingIterator};
fn main() {
    let mut journal = Journal::<u32, 3>::new();
    let mut windows = journal.windows_mut(2);
    let mut first = windows.next().unwrap();
    let mut second = windows.next().unwrap();
    println!("{:?} {:?}", first.get_mut(0), second.get_mut(0));
}
