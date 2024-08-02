use trustfall::{FieldValue, provider::{AsVertex, ContextIterator, ContextOutcomeIterator, ResolveInfo}};

use super::vertex::Vertex;

pub(super) fn resolve_decoded_instruction_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "address" => {
            todo!(
                "implement property 'address' in fn `resolve_decoded_instruction_property()`"
            )
        }
        "length" => {
            todo!(
                "implement property 'length' in fn `resolve_decoded_instruction_property()`"
            )
        }
        "name" => {
            todo!(
                "implement property 'name' in fn `resolve_decoded_instruction_property()`"
            )
        }
        "operands" => {
            todo!(
                "implement property 'operands' in fn `resolve_decoded_instruction_property()`"
            )
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'DecodedInstruction'"
            )
        }
    }
}

pub(super) fn resolve_source_location_property<'a, V: AsVertex<Vertex> + 'a>(
    contexts: ContextIterator<'a, V>,
    property_name: &str,
    _resolve_info: &ResolveInfo,
) -> ContextOutcomeIterator<'a, V, FieldValue> {
    match property_name {
        "column" => {
            todo!(
                "implement property 'column' in fn `resolve_source_location_property()`"
            )
        }
        "file" => {
            todo!("implement property 'file' in fn `resolve_source_location_property()`")
        }
        "line" => {
            todo!("implement property 'line' in fn `resolve_source_location_property()`")
        }
        _ => {
            unreachable!(
                "attempted to read unexpected property '{property_name}' on type 'SourceLocation'"
            )
        }
    }
}
