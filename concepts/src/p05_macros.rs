macro_rules! make_pair {
    ($value:expr) => {{
        // PUZZLE BEGIN demo
        todo!("p05_macros: demo")
        // PUZZLE END demo
    }};
}
macro_rules! repeat_vec {
    ($value:expr, $count:expr) => {{
        // PUZZLE BEGIN exercise
        todo!("p05_macros: exercise")
        // PUZZLE END exercise
    }};
}
#[cfg(test)]
#[allow(unreachable_code, unused_mut)]
mod tests {
    #[test]
    fn demo_side_effect_once() {
        let mut calls = 0;
        let pair: (i32, i32) = make_pair!({
            calls += 1;
            7
        });
        assert_eq!(pair, (7, 7));
        assert_eq!(calls, 1);
    }
    #[test]
    fn exercise_clone_and_evaluation() {
        let mut calls = 0;
        let values: Vec<String> = repeat_vec!(
            {
                calls += 1;
                String::from("x")
            },
            3
        );
        assert_eq!(values, vec!["x", "x", "x"]);
        assert_eq!(calls, 1);
        let empty: Vec<i32> = repeat_vec!(5, 0);
        assert!(empty.is_empty());
    }
}
