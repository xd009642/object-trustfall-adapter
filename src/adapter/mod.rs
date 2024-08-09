use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod adapter_impl;
mod edges;
mod entrypoints;
mod properties;
mod vertex;

#[cfg(test)]
mod tests;

pub use adapter_impl::Adapter;
pub use vertex::Vertex;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}
