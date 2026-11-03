//! Query a real Arrow RecordBatch, including independently nullable columns.
use arrow::{
    array::{Array, BooleanArray, UInt32Array, UInt64Array},
    compute::{and_kleene, cast, filter, sum},
    datatypes::{DataType, Field, Schema},
    error::ArrowError,
    record_batch::RecordBatch,
};
use std::sync::Arc;
#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub matched: usize,
    pub latency_us: u64,
}
fn columns(batch: &RecordBatch) -> Result<(&UInt32Array, &UInt32Array), ArrowError> {
    let get = |name| {
        batch
            .column_by_name(name)
            .and_then(|v| v.as_any().downcast_ref::<UInt32Array>())
            .ok_or_else(|| ArrowError::SchemaError(format!("missing UInt32 column {name}")))
    };
    Ok((get("latency_us")?, get("status")?))
}
/// Composable library plan: compare -> mask -> filter -> widen -> aggregate.
pub fn kernels(batch: &RecordBatch, threshold: u32) -> Result<Summary, ArrowError> {
    use arrow::compute::kernels::cmp::{gt_eq, lt_eq};
    let (latency, status) = columns(batch)?;
    let slow = gt_eq(latency, &UInt32Array::new_scalar(threshold))?;
    let lower = gt_eq(status, &UInt32Array::new_scalar(500))?;
    let upper = lt_eq(status, &UInt32Array::new_scalar(599))?;
    let mask: BooleanArray = and_kleene(&slow, &and_kleene(&lower, &upper)?)?;
    let selected = filter(latency, &mask)?;
    let widened = cast(&selected, &DataType::UInt64)?;
    let values = widened
        .as_any()
        .downcast_ref::<UInt64Array>()
        .expect("cast output type");
    // RecordBatch row count is bounded in the ingestion layer; check in both plans.
    if values.len() as u128 * u128::from(u32::MAX) > u128::from(u64::MAX) {
        return Err(ArrowError::ComputeError("sum bound".into()));
    }
    Ok(Summary {
        matched: values.len(),
        latency_us: sum(values).unwrap_or(0),
    })
}
/// Single-purpose aggregation avoids intermediate masks and filtered arrays.
/// It is scalar Rust; the compiler decides whether/how to vectorize.
pub fn fused(batch: &RecordBatch, threshold: u32) -> Result<Summary, ArrowError> {
    let (latency, status) = columns(batch)?;
    let mut result = Summary {
        matched: 0,
        latency_us: 0,
    };
    for (latency, status) in latency.iter().zip(status.iter()) {
        if let (Some(latency), Some(status)) = (latency, status) {
            if latency >= threshold && (500..=599).contains(&status) {
                result.matched += 1;
                result.latency_us = result
                    .latency_us
                    .checked_add(u64::from(latency))
                    .ok_or_else(|| ArrowError::ComputeError("sum overflow".into()))?;
            }
        }
    }
    Ok(result)
}
pub fn batch(latency: Vec<Option<u32>>, status: Vec<Option<u32>>) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("latency_us", DataType::UInt32, true),
        Field::new("status", DataType::UInt32, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt32Array::from(latency)),
            Arc::new(UInt32Array::from(status)),
        ],
    )
    .unwrap()
}
pub fn fixture(rows: usize, null_every: usize, error_every: usize) -> RecordBatch {
    assert!(null_every > 0 && error_every > 0);
    batch(
        (0..rows)
            .map(|i| (i % null_every != 0).then_some((i * 97 % 100_000) as u32))
            .collect(),
        (0..rows)
            .map(|i| {
                (i % (null_every + 1) != 0).then_some(if i % error_every == 0 { 503 } else { 200 })
            })
            .collect(),
    )
}
