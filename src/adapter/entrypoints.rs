use trustfall::provider::{ResolveInfo, VertexIterator};

use super::vertex::Vertex;

pub(super) fn debug_info<'a>(_resolve_info: &ResolveInfo) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'debug_info'")
}

pub(super) fn get_file_instructions<'a>(
    file: &str,
    _resolve_info: &ResolveInfo,
) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'getFileInstructions'")
}

pub(super) fn get_file_locations<'a>(
    file: &str,
    _resolve_info: &ResolveInfo,
) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'getFileLocations'")
}

pub(super) fn get_instruction<'a>(
    address: i64,
    _resolve_info: &ResolveInfo,
) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'getInstruction'")
}

pub(super) fn get_location<'a>(
    address: i64,
    _resolve_info: &ResolveInfo,
) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'getLocation'")
}

pub(super) fn text_section<'a>(_resolve_info: &ResolveInfo) -> VertexIterator<'a, Vertex> {
    todo!("implement resolving starting vertices for entrypoint edge 'text_section'")
}
