use case08c_simd::*;
type Scan = fn(Columns<'_>, Query) -> Result<Summary, Error>;
fn implementations() -> [Scan; 3] {
    [scan_unrolled, scan_simd, scan_auto]
}
#[test]
fn every_tail_length_and_selectivity() {
    for n in 0..=73 {
        for sparse in [false, true] {
            let c = fixture(n, sparse);
            for latency in [0, 1, 50_000, u32::MAX] {
                let q = Query {
                    min_latency: latency,
                    min_status: 500,
                };
                for f in implementations() {
                    assert_eq!(f(c.view(), q), scan_scalar(c.view(), q), "n={n}");
                }
            }
        }
    }
}
#[test]
fn widening_precedes_lane_reduction() {
    let latency = [u32::MAX; 17];
    let status = [599; 17];
    let c = Columns {
        latency: &latency,
        status: &status,
        validity: &[255, 255, 255],
    };
    for f in implementations() {
        assert_eq!(
            f(
                c,
                Query {
                    min_latency: u32::MAX,
                    min_status: 599
                }
            ),
            Ok(Summary {
                matched: 17,
                total_latency: 17 * u32::MAX as u64
            })
        );
    }
}
#[test]
fn nulls_and_padding_bits_are_ignored() {
    let latency = [u32::MAX; 9];
    let status = [503; 9];
    for validity in [[0, 254], [255, 0], [0, 1]] {
        let c = Columns {
            latency: &latency,
            status: &status,
            validity: &validity,
        };
        for f in implementations() {
            assert_eq!(
                f(
                    c,
                    Query {
                        min_latency: 0,
                        min_status: 500
                    }
                ),
                scan_scalar(
                    c,
                    Query {
                        min_latency: 0,
                        min_status: 500
                    }
                )
            );
        }
    }
}
#[test]
fn invalid_status_does_not_match() {
    let c = Columns {
        latency: &[100; 8],
        status: &[0, 99, 100, 499, 500, 599, 600, u32::MAX],
        validity: &[255],
    };
    for f in implementations() {
        assert_eq!(
            f(
                c,
                Query {
                    min_latency: 100,
                    min_status: 500
                }
            ),
            Ok(Summary {
                matched: 2,
                total_latency: 200
            })
        );
    }
}
#[test]
fn rejects_shape_before_scanning() {
    let q = Query {
        min_latency: 0,
        min_status: 500,
    };
    for f in implementations() {
        assert_eq!(
            f(
                Columns {
                    latency: &[1],
                    status: &[],
                    validity: &[255]
                },
                q
            ),
            Err(Error::Length)
        );
        assert_eq!(
            f(
                Columns {
                    latency: &[1],
                    status: &[200],
                    validity: &[]
                },
                q
            ),
            Err(Error::Bitmap)
        );
        assert_eq!(
            f(
                Columns {
                    latency: &[],
                    status: &[],
                    validity: &[0]
                },
                q
            ),
            Err(Error::Bitmap)
        );
        assert_eq!(
            f(
                Columns {
                    latency: &[],
                    status: &[],
                    validity: &[]
                },
                Query { min_status: 0, ..q }
            ),
            Err(Error::Query)
        );
    }
}
#[test]
fn offset_slices_need_no_vector_alignment() {
    let a: Vec<_> = (0..100).map(|x| x * 1000).collect();
    let s = vec![503; 101];
    let c = Columns {
        latency: &a[1..98],
        status: &s[2..99],
        validity: &[255; 13],
    };
    for f in implementations() {
        assert_eq!(
            f(
                c,
                Query {
                    min_latency: 8000,
                    min_status: 500
                }
            ),
            scan_scalar(
                c,
                Query {
                    min_latency: 8000,
                    min_status: 500
                }
            )
        );
    }
}
#[test]
fn deterministic_random_differential_and_dispatch_boundaries() {
    let mut seed = 17u64;
    for n in [127, 128, 129, 1027, 4099] {
        let mut c = fixture(n, false);
        for i in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            c.latency[i] = (seed >> 32) as u32;
            c.status[i] = ((seed >> 16) % 800) as u32;
        }
        for b in &mut c.validity {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *b = (seed >> 32) as u8;
        }
        let q = Query {
            min_latency: 1_000_000,
            min_status: 499,
        };
        for f in implementations() {
            assert_eq!(f(c.view(), q), scan_scalar(c.view(), q));
        }
    }
}
