use std::marker::PhantomData;
pub struct Closed;
pub struct Open;
pub struct Door<State> {
    name: String,
    state: PhantomData<State>,
}
impl Door<Closed> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            state: PhantomData,
        }
    }
    pub fn open(self) -> Door<Open> {
        // PUZZLE BEGIN demo
        todo!("p04_typestate: demo")
        // PUZZLE END demo
    }
}
impl Door<Open> {
    pub fn walk(&self) -> &str {
        &self.name
    }
    pub fn close(self) -> Door<Closed> {
        // PUZZLE BEGIN exercise
        todo!("p04_typestate: exercise")
        // PUZZLE END exercise
    }
}
/// A closed door has no walk operation.
/// ```compile_fail
/// use concept_puzzles::p04_typestate::Door;
/// let door = Door::new("front");
/// door.walk();
/// ```
pub fn state_rule() {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_open_preserves_name() {
        assert_eq!(Door::new("front").open().walk(), "front");
    }
    #[test]
    fn exercise_close_and_reopen() {
        let closed: Door<Closed> = Door::new("back").open().close();
        assert_eq!(closed.open().walk(), "back");
    }
}
