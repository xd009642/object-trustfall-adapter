use std::sync::Arc;
use trustfall::{
    provider::{AsVertex, ContextIterator, ContextOutcomeIterator, DataContext, ResolveInfo},
    FieldValue,
};

use super::vertex::Vertex;

pub(super) fn resolve_decoded_instruction_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    let func = match property_name {
        "address" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::DecodedInstruction(instr)) => (v.clone(), FieldValue::Uint64(instr.ip())),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "length" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::DecodedInstruction(instr)) => {
                (v.clone(), FieldValue::Uint64(instr.len() as u64))
            }
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "name" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::DecodedInstruction(instr)) => {
                let string = format!("{:?}", instr.mnemonic());
                (v.clone(), FieldValue::String(Arc::from(string.as_str())))
            }
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "operands" => {
            println!(
                "implement property 'operands' in fn `resolve_decoded_instruction_property()`"
            );
            |v: DataContext<V>| match v.active_vertex() {
                Some(Vertex::DecodedInstruction(instr)) => {
                    let operands = vec![];
                    (v.clone(), FieldValue::List(operands.into()));
                    todo!("Not a real operands")
                }
                None => (v, FieldValue::Null),
                Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
            }
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DecodedInstruction'"
            )
        }
    };
    Box::new(contexts.map(func))
}

pub(super) fn resolve_source_location_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    let func = match property_name {
        "column" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (v.clone(), FieldValue::Uint64(loc.column as u64)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "file" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (
                v.clone(),
                FieldValue::String(Arc::from(loc.file.display().to_string().as_str())),
            ),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "line" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::SourceLocation(loc)) => (v.clone(), FieldValue::Uint64(loc.line as u64)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SourceLocation'"
            )
        }
    };
    Box::new(contexts.map(func))
}
