use super::SourceLocation;
use iced_x86::Instruction;

#[non_exhaustive]
#[derive(Debug, Clone, trustfall::provider::TrustfallEnumVertex)]
pub enum Vertex {
    DecodedInstruction(Instruction),
    SourceLocation(SourceLocation),
}
