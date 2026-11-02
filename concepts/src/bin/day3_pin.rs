fn main() {
    // TASK_BEGIN demonstrate
    let mut boxed = Box::new(AddressSensitive { id: 7, _pin: PhantomPinned });
    let removed = std::mem::replace(&mut *boxed, AddressSensitive { id: 8, _pin: PhantomPinned });
    println!("ordinary Box: removed {}, now holds {}", removed.id, boxed.id);
    assert_eq!((removed.id, boxed.id), (7, 8));
    // TASK_END demonstrate
}

use std::marker::PhantomPinned;
struct AddressSensitive { id: u32, _pin: PhantomPinned }
