use super::SourceLocation;
use iced_x86::Instruction;
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug, Clone, trustfall::provider::TrustfallEnumVertex)]
pub enum Vertex {
    BasicBlock(()),
    DecodedInstruction(Rc<Instruction>),
    Function(()),
    SourceLocation(Rc<SourceLocation>),
}
