use super::SourceLocation;
use iced_x86::Instruction;
use std::rc::Rc;

#[non_exhaustive]
#[derive(Debug, Clone, trustfall::provider::TrustfallEnumVertex)]
pub enum Vertex {
    BasicBlock(Rc<BasicBlock>),
    DecodedInstruction(Rc<Instruction>),
    Function(Rc<Function>),
    SourceLocation(Rc<SourceLocation>),
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub base_address: u64,
    pub instructions: Vec<Rc<Instruction>>,
    pub children: Vec<Rc<BasicBlock>>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub address: u64,
    pub name: String,
    pub stack_size: u64,
    pub basic_blocks: Rc<BasicBlock>,
}
