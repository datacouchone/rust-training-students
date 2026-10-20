#![allow(dead_code)]
use lab5_puzzle::WireSummary;
use std::{cell::Cell, rc::Rc};

#[test]
fn summary_retains_declaration_order() {
    #[derive(WireSummary)]
    struct Event {
        sequence: u32,
        healthy: bool,
    }
    assert_eq!(
        Event {
            sequence: 7,
            healthy: true
        }
        .wire_summary(),
        "Event { sequence: 7, healthy: true }"
    );
}
#[test]
fn summary_formats_strings_with_debug() {
    #[derive(WireSummary)]
    struct Name {
        value: &'static str,
    }
    assert_eq!(
        Name { value: "a\nb" }.wire_summary(),
        "Name { value: \"a\\nb\" }"
    );
}
#[test]
fn summary_rename_is_data_not_format_source() {
    #[derive(WireSummary)]
    struct Named {
        #[summary(rename = "wire{key}\"")]
        value: u8,
    }
    assert_eq!(
        Named { value: 9 }.wire_summary(),
        "Named { wire{key}\": 9 }"
    );
}
#[test]
fn summary_skip_does_not_require_debug() {
    struct NoDebug;
    #[derive(WireSummary)]
    struct Hidden<T> {
        id: u32,
        #[summary(skip)]
        hidden: T,
    }
    assert_eq!(
        Hidden {
            id: 3,
            hidden: NoDebug
        }
        .wire_summary(),
        "Hidden { id: 3 }"
    );
}
#[test]
fn summary_redaction_never_calls_debug() {
    struct Explosive;
    impl std::fmt::Debug for Explosive {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("a redacted field must never be formatted");
        }
    }
    #[derive(WireSummary)]
    struct Login {
        user: u8,
        #[summary(redact)]
        token: Explosive,
    }
    assert_eq!(
        Login {
            user: 4,
            token: Explosive
        }
        .wire_summary(),
        "Login { user: 4, token: <redacted> }"
    );
}
#[test]
fn summary_redaction_also_accepts_non_debug_types() {
    struct NoDebug;
    #[derive(WireSummary)]
    struct Secret<T> {
        #[summary(rename = "credential")]
        #[summary(redact)]
        token: T,
    }
    assert_eq!(
        Secret { token: NoDebug }.wire_summary(),
        "Secret { credential: <redacted> }"
    );
}
#[test]
fn summary_preserves_lifetime_const_parameters_and_where_clause() {
    #[derive(WireSummary)]
    struct Packet<'a, T, const N: usize>
    where
        T: Copy,
    {
        owner: &'a str,
        bytes: [T; N],
    }
    let owner = String::from("node");
    let packet = Packet {
        owner: &owner,
        bytes: [1u8, 2, 3],
    };
    assert_eq!(
        packet.wire_summary(),
        "Packet { owner: \"node\", bytes: [1, 2, 3] }"
    );
}
#[test]
fn summary_preserves_generic_defaults() {
    #[derive(WireSummary)]
    struct Defaulted<T = u8, const N: usize = 2> {
        bytes: [T; N],
    }
    let packet: Defaulted = Defaulted { bytes: [8, 9] };
    assert_eq!(packet.wire_summary(), "Defaulted { bytes: [8, 9] }");
}
#[test]
fn summary_supports_empty_and_all_hidden_structs() {
    #[derive(WireSummary)]
    struct Empty {}
    #[derive(WireSummary)]
    struct Hidden {
        #[summary(skip)]
        secret: u32,
    }
    assert_eq!(Empty {}.wire_summary(), "Empty {}");
    assert_eq!(Hidden { secret: 77 }.wire_summary(), "Hidden {}");
}
#[test]
fn summary_unraws_default_field_labels() {
    #[derive(WireSummary)]
    struct Keywords {
        r#type: u8,
    }
    assert_eq!(
        Keywords { r#type: 5 }.wire_summary(),
        "Keywords { type: 5 }"
    );
}
#[test]
fn summary_does_not_clone_or_consume_visible_fields() {
    struct Counted(Rc<Cell<usize>>);
    impl std::fmt::Debug for Counted {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.set(self.0.get() + 1);
            f.write_str("counted")
        }
    }
    #[derive(WireSummary)]
    struct Owner {
        value: Counted,
    }
    let calls = Rc::new(Cell::new(0));
    let item = Owner {
        value: Counted(calls.clone()),
    };
    assert_eq!(item.wire_summary(), "Owner { value: counted }");
    assert_eq!(calls.get(), 1);
    assert_eq!(item.wire_summary(), "Owner { value: counted }");
    assert_eq!(calls.get(), 2);
}
#[test]
fn summary_is_hygienic_in_a_hostile_caller_scope() {
    mod hostile {
        #![allow(non_camel_case_types)]
        pub struct String;
        pub struct Vec;
        #[derive(lab5_puzzle::WireSummary)]
        pub struct Packet {
            pub id: u8,
        }
    }
    assert_eq!(hostile::Packet { id: 6 }.wire_summary(), "Packet { id: 6 }");
}
