#[allow(unused_imports)]
use day04_production::{routing::{self, Locked, Published, Snapshot}, encoding::{self, Encoder, Policy}, analytics};
#[allow(unused_imports)]
use std::sync::{Arc, RwLock};
fn main() {
    // TASK_BEGIN demonstrate
    use arrow::ipc::{reader::StreamReader, writer::StreamWriter};
    let original = analytics::fixture(4108, 17, 19);
    let batch = original;
    let mut bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut bytes, &batch.schema()).unwrap();
        writer.write(&batch).unwrap();
        writer.finish().unwrap();
    }
    let restored = StreamReader::try_new(std::io::Cursor::new(bytes), None).unwrap().next().unwrap().unwrap();
    assert_eq!(analytics::fused(&restored, 50_000).unwrap(), analytics::kernels(&batch, 50_000).unwrap());
    println!("IPC round trip: {} rows; aggregate preserved", restored.num_rows());
    // TASK_END demonstrate
}
