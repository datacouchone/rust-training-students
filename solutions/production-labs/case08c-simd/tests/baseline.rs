use case08c_simd::*;
#[test]
fn oracle_applies_validity_and_predicate() {
    let c = Columns {
        latency: &[1, 100, 300],
        status: &[503, 503, 503],
        validity: &[0b011],
    };
    assert_eq!(
        scan_scalar(
            c,
            Query {
                min_latency: 50,
                min_status: 500
            }
        ),
        Ok(Summary {
            matched: 1,
            total_latency: 100
        })
    );
}
