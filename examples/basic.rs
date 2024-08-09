use object_trustfall_adapter::adapter::Adapter;
use std::collections::BTreeMap;
use std::sync::Arc;
use trustfall::{execute_query, FieldValue};

fn main() -> anyhow::Result<()> {
    let object = Arc::new(Adapter::load("target/debug/examples/basic")?);

    let query = "
        {
            getFileLocations(file: \"examples/basic.rs\") {
                file,
                line @output,
                column,
            }
        }
        ";

    let variables: BTreeMap<Arc<str>, FieldValue> = BTreeMap::new(); // [("file", FieldValue::String("basic.rs".into()))].into_iter().collect();
                                                                     //let variables = [("file", FieldValue::String("basic.rs".into()))].into_iter().collect();

    let result = execute_query(Adapter::schema(), object.clone(), query, variables).unwrap();

    let lines = result.collect::<Vec<_>>();
    println!("Basic.rs lines: {:?}", lines);
    Ok(())
}
