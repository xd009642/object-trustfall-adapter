use std::sync::{Arc, OnceLock};

use trustfall::{
    provider::{
        resolve_coercion_using_schema, resolve_property_with, AsVertex, ContextIterator,
        ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, ResolveInfo, Typename,
        VertexIterator,
    },
    FieldValue, Schema,
};

use super::vertex::Vertex;

static SCHEMA: OnceLock<Schema> = OnceLock::new();

#[non_exhaustive]
#[derive(Debug)]
pub struct Adapter {}

impl Adapter {
    pub const SCHEMA_TEXT: &'static str = include_str!("./schema.graphql");

    pub fn schema() -> &'static Schema {
        SCHEMA.get_or_init(|| Schema::parse(Self::SCHEMA_TEXT).expect("not a valid schema"))
    }

    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> trustfall::provider::Adapter<'a> for Adapter {
    type Vertex = Vertex;

    fn resolve_starting_vertices(
        &self,
        edge_name: &Arc<str>,
        parameters: &EdgeParameters,
        resolve_info: &ResolveInfo,
    ) -> VertexIterator<'a, Self::Vertex> {
        match edge_name.as_ref() {
            "debug_info" => super::entrypoints::debug_info(resolve_info),
            "getFileInstructions" => {
                let file: &str = parameters
                    .get("file")
                    .expect(
                        "failed to find parameter 'file' when resolving 'getFileInstructions' starting vertices",
                    )
                    .as_str()
                    .expect(
                        "unexpected null or other incorrect datatype for Trustfall type 'String!'",
                    );
                super::entrypoints::get_file_instructions(file, resolve_info)
            }
            "getFileLocations" => {
                let file: &str = parameters
                    .get("file")
                    .expect(
                        "failed to find parameter 'file' when resolving 'getFileLocations' starting vertices",
                    )
                    .as_str()
                    .expect(
                        "unexpected null or other incorrect datatype for Trustfall type 'String!'",
                    );
                super::entrypoints::get_file_locations(file, resolve_info)
            }
            "getInstruction" => {
                let address: i64 = parameters
                    .get("address")
                    .expect(
                        "failed to find parameter 'address' when resolving 'getInstruction' starting vertices",
                    )
                    .as_i64()
                    .expect(
                        "unexpected null or other incorrect datatype for Trustfall type 'Int!'",
                    );
                super::entrypoints::get_instruction(address, resolve_info)
            }
            "getLocation" => {
                let address: i64 = parameters
                    .get("address")
                    .expect(
                        "failed to find parameter 'address' when resolving 'getLocation' starting vertices",
                    )
                    .as_i64()
                    .expect(
                        "unexpected null or other incorrect datatype for Trustfall type 'Int!'",
                    );
                super::entrypoints::get_location(address, resolve_info)
            }
            "text_section" => super::entrypoints::text_section(resolve_info),
            _ => {
                unreachable!(
                    "attempted to resolve starting vertices for unexpected edge name: {edge_name}"
                )
            }
        }
    }

    fn resolve_property<V: AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: ContextIterator<'a, V>,
        type_name: &Arc<str>,
        property_name: &Arc<str>,
        resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'a, V, FieldValue> {
        if property_name.as_ref() == "__typename" {
            return resolve_property_with(contexts, |vertex| vertex.typename().into());
        }
        match type_name.as_ref() {
            "DecodedInstruction" => super::properties::resolve_decoded_instruction_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            "SourceLocation" => super::properties::resolve_source_location_property(
                contexts,
                property_name.as_ref(),
                resolve_info,
            ),
            _ => {
                unreachable!(
                    "attempted to read property '{property_name}' on unexpected type: {type_name}"
                )
            }
        }
    }

    fn resolve_neighbors<V: AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: ContextIterator<'a, V>,
        type_name: &Arc<str>,
        edge_name: &Arc<str>,
        parameters: &EdgeParameters,
        resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'a, V, VertexIterator<'a, Self::Vertex>> {
        match type_name.as_ref() {
            _ => {
                unreachable!(
                    "attempted to resolve edge '{edge_name}' on unexpected type: {type_name}"
                )
            }
        }
    }

    fn resolve_coercion<V: AsVertex<Self::Vertex> + 'a>(
        &self,
        contexts: ContextIterator<'a, V>,
        _type_name: &Arc<str>,
        coerce_to_type: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'a, V, bool> {
        resolve_coercion_using_schema(contexts, Self::schema(), coerce_to_type.as_ref())
    }
}
