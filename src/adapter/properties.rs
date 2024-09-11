use super::vertex::Vertex;
use iced_x86::{Formatter, NasmFormatter};
use std::sync::Arc;
use trustfall::{
    provider::{AsVertex, ContextIterator, ContextOutcomeIterator, DataContext, ResolveInfo},
    FieldValue,
};

pub(super) fn resolve_basic_block_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    let func = match property_name {
        "base_address" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::BasicBlock(block)) => (v.clone(), FieldValue::Uint64(block.base_address)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'BasicBlock'"
            )
        }
    };
    Box::new(contexts.map(func))
}

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
                    let mut operands = String::new();
                    let mut fmt = NasmFormatter::new();
                    fmt.format_all_operands(instr, &mut operands);
                    let operands = operands
                        .split(",")
                        .map(|x| FieldValue::String(x.into()))
                        .collect::<Vec<_>>();
                    (v.clone(), FieldValue::List(operands.into()))
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

pub(super) fn resolve_function_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    let func = match property_name {
        "address" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::Function(func)) => (v.clone(), FieldValue::Uint64(func.address)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "name" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::Function(func)) => {
                (v.clone(), FieldValue::String(Arc::from(func.name.as_str())))
            }
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        "stack_size" => |v: DataContext<V>| match v.active_vertex() {
            Some(Vertex::Function(func)) => (v.clone(), FieldValue::Uint64(func.stack_size)),
            None => (v, FieldValue::Null),
            Some(vertex) => unreachable!("Invalid vertex: {:?}", vertex),
        },
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'Function'"
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
