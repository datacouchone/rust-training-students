#[derive(Debug, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
}
pub fn parse_metric(raw: &str) -> Result<Metric, &'static str> {
    // PUZZLE BEGIN demo
    todo!("p09_debugging: demo")
    // PUZZLE END demo
}
pub fn decode_frame(bytes: &[u8]) -> Result<String, &'static str> {
    // PUZZLE BEGIN exercise
    todo!("p09_debugging: exercise")
    // PUZZLE END exercise
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_good_and_malformed_metric() {
        assert_eq!(
            parse_metric("load=0.5"),
            Ok(Metric {
                name: "load".into(),
                value: 0.5
            })
        );
        for invalid in ["", "load", "=1", "load=x", "load=NaN", "load=inf"] {
            assert!(parse_metric(invalid).is_err());
        }
        assert!(parse_metric("delta=-1").is_ok());
    }
    #[test]
    fn exercise_framing_and_bad_utf8() {
        assert_eq!(decode_frame(&[0, 0, 0, 2, b'O', b'K']), Ok("OK".into()));
        assert_eq!(decode_frame(&[0, 0, 0, 0]), Ok(String::new()));
        for invalid in [
            &[0, 0][..],
            &[0, 0, 0, 2, b'O'][..],
            &[0, 0, 0, 1, 255][..],
            &[0, 0, 0, 0, 1][..],
        ] {
            assert!(decode_frame(invalid).is_err());
        }
    }
}
