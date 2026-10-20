#![allow(dead_code)]
use lab5_puzzle::WireSummary;

#[derive(WireSummary)]
struct Envelope<'a, T, const N: usize>
where
    T: Clone,
{
    name: &'a str,
    data: [T; N],
}
#[test]
fn derive_preserves_the_original_type_and_fields() {
    let packet = Envelope {
        name: "events",
        data: [1u32, 2],
    };
    assert_eq!(packet.name, "events");
    assert_eq!(packet.data, [1, 2]);
}
#[test]
fn a_generic_declaration_can_be_instantiated() {
    let _: Envelope<'_, String, 0> = Envelope {
        name: "empty",
        data: [],
    };
}
