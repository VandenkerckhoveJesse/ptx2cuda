use std::num::NonZeroI8;
use ptx_syntax::r#type::{Module, ModuleDirective, FunctionStatement, Instruction};
use ptx_syntax::r#type::instruction::Inst;
use crate::ir::{Kernel, Quad, Opcode, Operand};

pub fn lift(ast: Module) -> Result<Kernel, String> {
    let entry = ast.directives.into_iter().find_map(|d| match d {
        ModuleDirective::EntryFunction {directive, ..} => Some(directive), _ => None,
    }).ok_or("no entry function")?;

    let mut quads: Vec<Quad> = Vec::new();
    for statement in entry.body.unwrap().statements {
        let quad = lift_quad(statement)?;
        quads.push(quad);
    }



    return Ok(Kernel{
        name: entry.name.val,
        quads: quads,
    })
}

fn lift_quad(statement: FunctionStatement) -> Result<Quad, String> {
    if let FunctionStatement::Instruction { instruction, .. } = statement {
        if let Inst::RetUni(_) = instruction.inst {
            Ok(Quad {
                opcode: Opcode::Ret,
                dest: Operand::None,
                src1: Operand::None,
                src2: Operand::None,
            })
        } else {
            Err("Instruction not return".into())
        }
    } else {
        Err("State not instruction".into())
    }
}

#[cfg(test)]
mod tests {
    use super::lift;
    use ptx_syntax::parse_ptx;
    use crate::ir::Opcode;

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
}
