use ffi_puzzle::{DbError, LegacyDb};
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn absence_empty_and_unicode_are_distinct() {
    let mut db = LegacyDb::open("values.db").unwrap();
    assert_eq!(db.get_str(b"missing"), Ok(None));
    db.put(b"", "").unwrap();
    assert_eq!(db.get_str(b""), Ok(Some("")));
    db.put("clé".as_bytes(), "こんにちは").unwrap();
    assert_eq!(db.get_str("clé".as_bytes()), Ok(Some("こんにちは")));
}

#[test]
fn borrowed_lookup_has_stable_address_without_copying() {
    let mut db = LegacyDb::open("address.db").unwrap();
    db.put(b"k", "a value longer than an empty string").unwrap();
    let first = db.get_str(b"k").unwrap().unwrap();
    let second = db.get_str(b"k").unwrap().unwrap();
    assert_eq!(first.as_ptr(), second.as_ptr());
    assert_eq!(first, second);
}

#[test]
fn output_borrows_database_not_temporary_key() {
    let mut db = LegacyDb::open("temporary-key.db").unwrap();
    db.put(b"k", "value").unwrap();
    let value = {
        let key = vec![b'k'];
        db.get_str(&key).unwrap().unwrap()
    };
    assert_eq!(value, "value");
    // This last use ends the borrow: mutation is now permitted.
    db.put(b"k", "replacement").unwrap();
    assert_eq!(db.get_str(b"k"), Ok(Some("replacement")));
}

#[test]
fn update_existing_and_missing_values_calls_once_each() {
    let mut db = LegacyDb::open("update.db").unwrap();
    let calls = Cell::new(0);
    db.update_with(b"k", |old| {
        calls.set(calls.get() + 1);
        assert_eq!(old, None);
        String::from("first")
    }).unwrap();
    db.update_with(b"k", |old| {
        calls.set(calls.get() + 1);
        assert_eq!(old, Some("first"));
        format!("{}-second", old.unwrap())
    }).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(db.get_str(b"k"), Ok(Some("first-second")));
}

#[test]
fn validation_precedence_and_callback_not_called_on_bad_keys() {
    let mut db = LegacyDb::open("invalid.db").unwrap();
    let bad_both: &[u8] = &[0xff, 0];
    assert_eq!(db.get_str(bad_both), Err(DbError::InvalidUtf8));
    assert_eq!(db.get_str(b"has\0nul"), Err(DbError::InteriorNul));
    for (key, expected) in [
        (bad_both, DbError::InvalidUtf8),
        (&b"has\0nul"[..], DbError::InteriorNul),
    ] {
        let calls = Cell::new(0);
        let result = db.update_with(key, |_| {
            calls.set(calls.get() + 1);
            String::from("must not be written")
        });
        assert_eq!(result, Err(expected));
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn invalid_output_leaves_existing_and_missing_keys_unchanged() {
    let mut db = LegacyDb::open("nul-output.db").unwrap();
    db.put(b"k", "original").unwrap();
    for key in [&b"k"[..], &b"missing"[..]] {
        let calls = Cell::new(0);
        assert_eq!(db.update_with(key, |_| {
            calls.set(calls.get() + 1);
            String::from("bad\0output")
        }), Err(DbError::InteriorNul));
        assert_eq!(calls.get(), 1);
    }
    assert_eq!(db.get_str(b"k"), Ok(Some("original")));
    assert_eq!(db.get_str(b"missing"), Ok(None));
}

#[test]
fn callback_panic_propagates_without_database_mutation() {
    let mut db = LegacyDb::open("panic.db").unwrap();
    db.put(b"k", "original").unwrap();
    let calls = Cell::new(0);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = db.update_with(b"k", |old| {
            calls.set(calls.get() + 1);
            assert_eq!(old, Some("original"));
            panic!("transform failed");
        });
    }));
    assert!(result.is_err(), "do not swallow the transform panic");
    assert_eq!(calls.get(), 1);
    assert_eq!(db.get_str(b"k"), Ok(Some("original")));
    db.put(b"k", "still usable").unwrap();
}

#[test]
fn transform_may_be_fnonce_and_return_empty_text() {
    let mut db = LegacyDb::open("once.db").unwrap();
    let owned = String::from("moved into callback");
    db.update_with(b"k", move |_| owned).unwrap();
    assert_eq!(db.get_str(b"k"), Ok(Some("moved into callback")));
    db.update_with(b"", |_| String::new()).unwrap();
    assert_eq!(db.get_str(b""), Ok(Some("")));
}

#[test]
fn owned_snapshot_survives_replacement_and_close() {
    let mut db = LegacyDb::open("snapshot.db").unwrap();
    db.put(b"k", "before").unwrap();
    let snapshot = db.get(b"k").unwrap();
    db.update_with(b"k", |old| format!("{}-after", old.unwrap())).unwrap();
    drop(db);
    assert_eq!(snapshot, "before");
}

#[test]
fn repeated_updates_preserve_values_across_replacement() {
    let mut db = LegacyDb::open("repeat.db").unwrap();
    for n in 0..32 {
        db.update_with(b"counter", |old| {
            let previous = old.map(|s| s.parse::<usize>().unwrap()).unwrap_or(0);
            assert_eq!(previous, n);
            (previous + 1).to_string()
        }).unwrap();
    }
    assert_eq!(db.get_str(b"counter"), Ok(Some("32")));
}
