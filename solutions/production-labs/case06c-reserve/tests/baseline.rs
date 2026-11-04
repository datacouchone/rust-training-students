use case06c_reserve::*;
#[test]
fn configuration_and_empty_snapshot() {
    assert_eq!(
        Config {
            capacity: 0,
            critical_reserve: 0
        }
        .validate(),
        Err(Error::InvalidConfig)
    );
    let p = Pool::new(Config {
        capacity: 10,
        critical_reserve: 3,
    })
    .unwrap();
    assert_eq!(
        p.snapshot(),
        Snapshot {
            config: Config {
                capacity: 10,
                critical_reserve: 3
            },
            normal: 0,
            critical: 0,
            closed: false
        }
    );
}
