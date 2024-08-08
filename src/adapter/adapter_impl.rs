use super::vertex::Vertex;
use super::SourceLocation;
use crate::loader::*;
use anyhow::Context;
use gimli::*;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use object::{read::ObjectSection, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use trustfall::{
    provider::{
        resolve_coercion_using_schema, resolve_property_with, AsVertex, ContextIterator,
        ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, ResolveInfo, Typename,
        VertexIterator,
    },
    FieldValue, Schema,
};

static SCHEMA: OnceLock<Schema> = OnceLock::new();

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Adapter {
    pub debug_info: BTreeMap<u64, Vec<Rc<SourceLocation>>>, // Address to code region
    pub text_section: Vec<Rc<Instruction>>,
}

impl Adapter {
    pub const SCHEMA_TEXT: &'static str = include_str!("./schema.graphql");

    pub fn schema() -> &'static Schema {
        SCHEMA.get_or_init(|| Schema::parse(Self::SCHEMA_TEXT).expect("not a valid schema"))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        let file = object::File::parse(&*data)?;

        let debug_info = match get_line_addresses(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("No debug info: {}", e);
                Default::default()
            }
        };

        let mut text_section = vec![];
        for section in file.sections() {
            let name = match section.name() {
                Ok(s) => s,
                Err(_e) => continue,
            };
            if name == ".text" {
                let bytes = section.data()?;
                let mut decoder = Decoder::new(64, bytes, DecoderOptions::NONE);
                text_section = decoder.iter().map(Rc::new).collect();
            }
        }
        Ok(Self {
            debug_info,
            text_section,
        })
    }

    pub fn find_instruction(&self, address: u64) -> Option<Rc<Instruction>> {
        self.text_section
            .iter()
            .find(|x| x.ip() >= address && address < (x.ip() - x.len() as u64))
            .cloned()
    }

    pub fn get_file_locations(&self, path: PathBuf) -> Vec<Rc<SourceLocation>> {
        self.debug_info
            .values()
            .flat_map(|x| x.iter().filter(|y| y.file == path).cloned())
            .collect()
    }

    pub fn get_file_instructions(&self, path: PathBuf) -> Vec<Rc<Instruction>> {
        let iter = self
            .debug_info
            .iter()
            .filter(|(_k, x)| x.iter().any(|y| y.file == path))
            .map(|(k, _)| k);

        iter.filter_map(|x| self.find_instruction(*x)).collect()
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
                let it = self
                    .get_file_instructions(file.into())
                    .into_iter()
                    .map(|x| Vertex::DecodedInstruction(x));
                Box::new(it)
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
                let it = self
                    .get_file_locations(file.into())
                    .into_iter()
                    .map(|x| Vertex::SourceLocation(x));
                Box::new(it)
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
                let instruction = self
                    .find_instruction(address as u64)
                    .map(|x| Vertex::DecodedInstruction(x));
                Box::new(instruction.into_iter())
            }
            "getLocation" => {
                let address = parameters
                    .get("address")
                    .expect(
                        "failed to find parameter 'address' when resolving 'getLocation' starting vertices",
                    )
                    .as_i64()
                    .expect(
                        "unexpected null or other incorrect datatype for Trustfall type 'Int!'",
                    ) as u64;
                match self.debug_info.get(&address) {
                    Some(val) => {
                        Box::new(val.clone().into_iter().map(|x| Vertex::SourceLocation(x)))
                    }
                    None => Box::new(std::iter::empty()),
                }
            }
            "text_section" => {
                let text_section = self.text_section.clone();
                let iter = text_section
                    .into_iter()
                    .map(|x| Vertex::DecodedInstruction(x.clone()));
                Box::new(iter)
            }
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
