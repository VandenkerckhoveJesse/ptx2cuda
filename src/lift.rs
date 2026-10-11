// Part of front end of decompiler, lifts parsed AST code into the intermediary representation.

use std::os::macos::raw::stat;
use std::vec::IntoIter;
use crate::ir::{Address, Comparison, Dim, Guard, Immediate, Instruction, Kernel, Label, Operand, Register, SpecialReg, StateSpace, Statement, Type};
use ptx_syntax::r#type::instruction::Inst;
use ptx_syntax::r#type::instruction::mov::MovType;
use ptx_syntax::r#type::instruction::setp::SetpCmpopFtzType;
use ptx_syntax::r#type::instruction::setp::section_0::Cmpop as PtxCmpop;
use ptx_syntax::r#type::{FunctionStatement, GeneralOperand, Instruction as PtxInstruction, Module, ModuleDirective, Operand as PtxOperand, Predicate as PtxPredicate, Immediate as PtxImmediate, AddressOperand, AddressBase};
use ptx_syntax::r#type::instruction::bra::BraUni;
use ptx_syntax::r#type::instruction::mov::section_0::Type as PtxType;
use ptx_syntax::r#type::instruction::st::section_0::{Ss, Type as StType};

pub fn lift(ast: Module) -> Result<Kernel, String> {
    let entry = ast
        .directives
        .into_iter()
        .find_map(|d| match d {
            ModuleDirective::EntryFunction { directive, .. } => Some(directive),
            _ => None,
        })
        .ok_or("unsupported PTX module: no entry function")?;

    let body = entry
        .body
        .ok_or_else(|| "unsupported entry function: it has no body")?;

    let mut statements: Vec<Statement> = Vec::new();
    let mut iterator = body.statements.into_iter();

    while let Some(statement) = lift_statement(&mut iterator)? {
        statements.push(statement);
    }

    Ok(Kernel(statements))
}

fn lift_statement(
    iter: &mut IntoIter<FunctionStatement>
) -> Result<Option<Statement>, String> {
    loop {
        let Some(item) = iter.next() else {
            return Ok(None)
        };
        let (label, item) = match item {
            FunctionStatement::Label { label, .. } => {
                let next = iter
                    .next()
                    .ok_or("label without following statement")?;

                (Some(label.val), next)
            }

            item => (None, item),
        };
        match item {
            FunctionStatement::Instruction { instruction: PtxInstruction {
                predicate,
                inst,
                ..
            }, ..
            } => return Ok(Some(Statement(label, lift_guard(predicate)?, lift_instruction(inst)?))),
            FunctionStatement::Directive { .. } if label.is_none() => continue, // Ignore for now, eventually will have to be put in symbol table
            FunctionStatement::Directive { .. } => {
                return Err("labelled directives are unsupported".into());
            }
            unsupported => {
                return Err(format!(
                    "unsupported PTX function statement: {unsupported:?}"
                ));
            }
        }
    }
}

fn lift_guard(predicate: Option<PtxPredicate>) -> Result<Option<Guard>, String> {
    let Some(PtxPredicate { negated, operand: PtxOperand::Register {operand: register, .. }, .. }) = predicate else {
        return Ok(None);
    };
    Ok(Some(Guard(negated, Register::Predicate(register.name))))
}

fn lift_instruction(inst: Inst) -> Result<Instruction, String> {
    match inst {
        Inst::BraUni(BraUni {uni, tgt, .. }) => {
            Ok(Instruction::Bra(lift_label(tgt)?))
        },
        Inst::RetUni(_) => Ok(Instruction::Ret),
        Inst::MovType(MovType { type_, d, a, .. }) => {
            let Operand::Register(destination) = lift_operand(d)? else {
                return Err("mov destination must be a register".into());
            };
            Ok(Instruction::Mov(lift_type(type_)?, destination, lift_operand(a)?))
        },
        Inst::SetpCmpopFtzType(SetpCmpopFtzType{ cmpop, ftz, type_, p, q, a, b, span }) => {
            let Operand::Register(destination) = lift_operand(p)? else {
                return Err("mov destination must be a register".into());
            };
            Ok(Instruction::Setp(lift_comparison(cmpop)?, lift_type(type_)?, destination, lift_operand(a)?, lift_operand(b)?))
        },
        Inst::StWeakSsCopLevelCacheHintVecType(StWeakSsCopLevelCacheHintVecType{weak, ss, cop, level_cache_hint, vec, type_, a, b, ..}) => {
            Ok(Instruction::St(lift_statespace(ss)?, lift_type(type_)?, lift_address(a)?, lift_operand(b)?))
        }

        unsupported => Err(format!("unsupported PTX instruction: {unsupported:?}")),
    }
}

fn lift_statespace(ss: Option<Ss>) -> Result<StateSpace, String> {
    let Some(Ss::Global) = ss else {
        return Err("only global statespace supported".into())
    };
    Ok(StateSpace::Global)
}

trait LiftType {
    fn lift(self) -> Result<Type, String>;
}

use ptx_syntax::r#type::instruction::mov::section_0::Type as MovPtxType;
use ptx_syntax::r#type::instruction::setp::section_0::Type as SetpPtxType;
use ptx_syntax::r#type::instruction::st::StWeakSsCopLevelCacheHintVecType;

impl LiftType for MovPtxType {
    fn lift(self) -> Result<Type, String> {
        match self {
            PtxType::U32 => Ok(Type::U32),
            PtxType::S32 => Ok(Type::S32),
            unsupported => Err(format!("unsupported PTX type: {unsupported:?}"))
        }
    }
}

impl LiftType for SetpPtxType {
    fn lift(self) -> Result<Type, String> {
        match self {
            SetpPtxType::S32 => Ok(Type::S32),
            SetpPtxType::U32 => Ok(Type::U32),
            unsupported => Err(format!("unsupported PTX type: {unsupported:?}"))
        }
    }
}

impl LiftType for StType {
    fn lift(self) -> Result<Type, String> {
        match self {
            StType::S32 => Ok(Type::S32),
            StType::U32 => Ok(Type::U32),
            unsupported => Err(format!("unsupported PTX type: {unsupported:?}"))
        }
    }
}

fn lift_type<T: LiftType>(typ: T) -> Result<Type, String> {
    typ.lift()
}

fn lift_operand(operand: GeneralOperand) -> Result<Operand, String> {
    match operand {
        GeneralOperand::Single {
            operand: PtxOperand::Register {
                operand: register, ..
            },
            ..
        } => {
            if register.name == "%tid" {
                let dimension = match register.component.as_deref() {
                    Some("x") => Dim::X,
                    Some("y") => Dim::Y,
                    Some("z") => Dim::Z,
                    other => {
                        return Err(format!("unsupported %tid component: {other:?}"))
                    }
                };
                return Ok(Operand::Register(Register::Special(SpecialReg::Tid(dimension))))
            }
            Ok(Operand::Register(Register::Normal(register.name)))
        },
        GeneralOperand::Single {
            operand: PtxOperand::Immediate {
                operand: immediate, ..
            },
            ..
        } => Ok(Operand::Immediate(lift_immediate(immediate)?)),
        unsupported => Err(format!("unsupported operand: {unsupported:?}")),
    }
}

fn lift_label(operand: GeneralOperand) -> Result<Label, String> {
    let GeneralOperand::Single {operand: PtxOperand::Symbol {name, ..}, .. } = operand else {
        return Err("expected a symbol operand".into())
    };
    Ok(name)
}

fn lift_address(operand: AddressOperand) -> Result<Address, String> {
    match operand {
        AddressOperand::Offset {
            base: AddressBase::Variable { symbol, .. },
            offset: None,
            ..
        } => Ok(Address(symbol.val)),

        other => Err(format!("unsupported address: {other:?}")),
    }
}


fn lift_immediate(immediate: PtxImmediate) -> Result<Immediate, String> {
    let value = immediate.value
        .parse::<i64>()
        .map_err(|error| error.to_string())?;

    Ok(Immediate::Int(value))
}

fn lift_comparison(comparison: PtxCmpop) -> Result<Comparison, String> {
    match comparison {
        PtxCmpop::Ne => Ok(Comparison::Ne),
        unsupported => Err(format!("unsupported comparison: {unsupported:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::lift;
    use crate::ir::{Address, Comparison, Dim, Guard, Immediate, Instruction, Kernel, Operand, Register, SpecialReg, StateSpace, Statement, Type};
    use ptx_syntax::parse_ptx;

    #[test]
    fn lifts_noop_program() {
        let source = include_str!("../tests/cuda/noop/noop.ptx");
        let module = parse_ptx(source).expect("noop.ptx should parse");

        let Kernel(statements) = lift(module).expect("lifting failed");

        assert_eq!(statements.len(), 1);
        let return_statement = statements.first().expect("expected at least one quad");
        assert_eq!(return_statement, &Statement(None, None, Instruction::Ret));
    }

    #[test]
    fn lifts_conditional_program() {
        let source = include_str!("../tests/cuda/conditional/conditional.ptx");
        let module = parse_ptx(source).expect("conditional.ptx should parse");

        let Kernel(statements) = lift(module).expect("lifting failed");

        assert_eq!(statements.len(), 6);
        assert_eq!(
            statements[0],
            Statement(
                None,
                None,
                Instruction::Mov(
                    Type::U32,
                    Register::Normal("%r1".into()),
                    Operand::Register(Register::Special(SpecialReg::Tid(Dim::X))),
                ),
            )
        );
        assert_eq!(
            statements[1],
            Statement(
                None,
                None,
                Instruction::Setp(
                    Comparison::Ne,
                    Type::S32,
                    Register::Normal("%p1".into()),
                    Operand::Register(Register::Normal("%r1".into())),
                    Operand::Immediate(Immediate::Int(0)),
                ),
            )
        );
        assert_eq!(
            statements[2],
            Statement(
                None,
                Some(Guard(false, Register::Predicate("%p1".into()))),
                Instruction::Bra("$L__BB0_2".into()),
            )
        );
        assert_eq!(
            statements[3],
            Statement(
                None,
                None,
                Instruction::Mov(
                    Type::U32,
                    Register::Normal("%r2".into()),
                    Operand::Immediate(Immediate::Int(1)),
                ),
            )
        );
        assert_eq!(
            statements[4],
            Statement(
                None,
                None,
                Instruction::St(
                    StateSpace::Global,
                    Type::U32,
                    Address("g".into()),
                    Operand::Register(Register::Normal("%r2".into())),
                ),
            )
        );
        assert_eq!(
            statements[5],
            Statement(Some("$L__BB0_2".into()), None, Instruction::Ret)
        );
    }
}
