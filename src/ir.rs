use std::collections::HashMap;

// Types
type Nat = usize;

pub type Label = String;

// Shared
#[derive(Debug, PartialEq)]
pub enum Type {U32, S32}

// Kernel In Vec<Block> blocks are represented in order, first is entry. HashMap<Label, usize> links label to index of block.
pub struct Kernel(pub Vec<Block>, pub HashMap<Label, usize>);

pub struct Block (pub Vec<Statement>);

#[derive(Debug, PartialEq)]
pub struct  Statement (pub Option<Guard>, pub Instruction);

// Predicate
#[derive(Debug, PartialEq)]
pub struct Guard(pub bool, pub Register);

// Instruction
#[derive(Debug, PartialEq)]
pub enum Instruction {
    Bra(Label),
    Mov(Type, Register, Operand),
    Setp(Comparison, Type, Register, Operand, Operand),
    St(StateSpace, Type, Address, Operand),
    Ret,
}

#[derive(Debug, PartialEq)]
pub enum Comparison {
    Ne
}

#[derive(Debug, PartialEq)]
pub enum StateSpace {
    Global
}

// Operand
#[derive(Debug, PartialEq)]
pub enum Operand {
    Register(Register),
    Immediate(Immediate)
    // review: should label also be part of operand, according to PTX spec its part of it, but only limited instructions use it
}

#[derive(Debug, PartialEq)]
pub enum Immediate {
    Int(i64)
}

#[derive(Debug, PartialEq)]
pub struct Address(pub String);

// Register
#[derive(Debug, PartialEq)]
pub enum Register {
    Normal(String),         // %r1  -> Normal("r", 1)
    Predicate(String),      // %p1  -> Predicate("p", 1)
    Special(SpecialReg),  // %tid.x -> Special(Tid, X)
}
#[derive(Debug, PartialEq)]
pub enum SpecialReg {
    Tid(Dim)
}
#[derive(Debug, PartialEq)]
pub enum Dim { X, Y, Z }





/*
type Name = String;
type Nat = usize;

enum Register {
    Predicate(Name, Nat),
    Normal(Name, Nat),
    Special(),
}

struct Statement(Option<Predicate>, Instruction);

enum Instruction {Mov(),}

enum Predicate {P()}

fn main() {
    let s = Statement(Some(P()),Mov() )
}*/
