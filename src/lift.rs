// Part of front end of decompiler, lifts parsed AST code into the intermediary representation.

use ptx_syntax::r#type::instruction::Inst;
use ptx_syntax::r#type::instruction::mov::MovType;
use ptx_syntax::r#type::{
    FunctionStatement, GeneralOperand, Module, ModuleDirective, Operand as PtxOperand,
};
use ptx_syntax::r#type::instruction::setp::section_0::Cmpop;
use ptx_syntax::r#type::instruction::setp::SetpCmpopFtzType;
use crate::ir::{Comparison, Kernel, Opcode, Operand, Quad};

pub fn lift(ast: Module) -> Result<Kernel, String> {
    let entry = ast
        .directives
        .into_iter()
        .find_map(|d| match d {
            ModuleDirective::EntryFunction { directive, .. } => Some(directive),
            _ => None,
        })
        .ok_or("unsupported PTX module: no entry function")?;

    let name = entry.name.val;
    let body = entry
        .body
        .ok_or_else(|| format!("unsupported entry function `{name}`: it has no body"))?;

    let mut quads: Vec<Quad> = Vec::new();
    for statement in body.statements {
        if let Some(quad) = lift_quad(statement)? {
            quads.push(quad)
        }
    }

    Ok(Kernel { name, quads })
}

fn lift_quad(statement: FunctionStatement) -> Result<Option<Quad>, String> {
    match statement {
        FunctionStatement::Instruction { instruction, .. } => match instruction.inst {
            Inst::RetUni(_) => Ok(Some(Quad {
                opcode: Opcode::Ret,
                dest: Operand::None,
                src1: Operand::None,
                src2: Operand::None,
            })),
            Inst::MovType(MovType { d, a, .. }) => Ok(Some(Quad {
                opcode: Opcode::Mov,
                dest: lift_operand(d)?,
                src1: lift_operand(a)?,
                src2: Operand::None,
            })),
            Inst::SetpCmpopFtzType(SetpCmpopFtzType{ cmpop, ftz, type_, p, q, a, b, span }) => Ok(Some(Quad {
                opcode: Opcode::Setp(lift_comparison(cmpop)?),
                dest: lift_operand(p)?,
                src1: lift_operand(a)?,
                src2: lift_operand(b)?,
            })),
            unsupported => Err(format!("unsupported PTX instruction: {unsupported:?}")),
        },
        FunctionStatement::Directive { .. } => Ok(None), // Ignore for now, eventually will have to be put in symbol table
        unsupported => Err(format!(
            "unsupported PTX function statement: {unsupported:?}"
        )),
    }
}

fn lift_operand(operand: GeneralOperand) -> Result<Operand, String> {
    match operand {
        GeneralOperand::Single {
            operand: PtxOperand::Register {
                operand: register, ..
            },
            ..
        } => Ok(Operand::Register {
            name: register.name,
            component: register.component,
        }),
        GeneralOperand::Single {
            operand: PtxOperand::Immediate {
                operand: immediate, ..
            },
            ..
        } => Ok(Operand::Immediate(immediate.value)),
        unsupported => Err(format!("unsupported operand: {unsupported:?}")),
    }
}

fn lift_comparison(comparison: Cmpop) -> Result<Comparison, String> {
    match comparison {
        Cmpop::Ne => Ok(Comparison::Ne),
        unsupported => Err(format!("unsupported comparison: {unsupported:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::lift;
    use crate::ir::Opcode;
    use ptx_syntax::parse_ptx;

    #[test]
    fn lifts_noop_program() {
        let source = include_str!("../tests/cuda/noop/noop.ptx");
        let module = parse_ptx(source).expect("noop.ptx should parse");

        let ir = lift(module).expect("lifting failed");

        assert_eq!(ir.quads.len(), 1);
        assert_eq!(ir.name, "_Z4noopv");
        let return_quad = ir.quads.first().expect("expected at least one quad");
        assert_eq!(return_quad.opcode, Opcode::Ret);
    }

    #[test]
    fn lifts_conditional_program() {
        let source = include_str!("../tests/cuda/conditional/conditional.ptx");
        let module = parse_ptx(source).expect("conditional.ptx should parse");

        let ir = lift(module).expect("lifting failed");

        assert_eq!(ir.quads.len(), 1);
        assert_eq!(ir.name, "_Z11conditionalv");
    }
}
