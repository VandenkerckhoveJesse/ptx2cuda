// Implements a Low-Level Intermediate Code as described by Ciffuentes 1994 but for PTX

pub struct Kernel {
    pub name: String,
    pub quads: Vec<Quad>,
}
pub struct Quad {
    pub opcode: Opcode,
    pub dest: Operand,
    pub src1: Operand,
    pub src2: Operand,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    Ret,
}

pub enum Operand {
    None,
}