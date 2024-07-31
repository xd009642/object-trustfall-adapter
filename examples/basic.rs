use object_trustfall_adapter::loader::ObjectFile;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> anyhow::Result<()> {
    let object = ObjectFile::load("target/debug/examples/basic")?;
    let file = File::create("object.json")?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &object)?;
    writer.flush()?;
    Ok(())
}
