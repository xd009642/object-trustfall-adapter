use object_trustfall_adapter::adapter::Adapter;
use std::sync::Arc;
use trustfall::{execute_query, FieldValue};

fn main() {
    let file = std::env::args().nth(1).expect("no object file given");
    let addr = std::env::args()
        .nth(2)
        .expect("no address given")
        .parse::<u64>()
        .expect("invalid address given");

    let object = Arc::new(Adapter::load(file).expect("Couldn't load file"));

    let query = "
        {
            getLocation(address: $addr) {
                file @output,
                line @output,
                column @output,
            }
        }
        ";

    let variables = [("addr", FieldValue::Int64(addr as i64))]
        .into_iter()
        .collect();
    println!("{:?}", variables);

    let result = execute_query(Adapter::schema(), object.clone(), query, variables).unwrap();

    let lines = result.collect::<Vec<_>>();
    if lines.is_empty() {
        panic!("No line for given address");
    } else {
        println!("{:?}", lines[0]);
    }
}
