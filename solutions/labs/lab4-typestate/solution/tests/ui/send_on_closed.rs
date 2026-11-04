// `send` only exists in `impl Socket<Connected>`, so calling it on a
// `Socket<Closed>` must be rejected at compile time — that's the whole
// point of the typestate pattern. This file must NOT compile.

use lab4_typestate_solution::{Closed, Socket};

fn main() {
    let mut socket = Socket::<Closed>::new();
    socket.send(b"nope");
}
