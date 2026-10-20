use genco::prelude::*;
use crate::cuda::*;

impl Type {
    fn to_str(&self) -> &str {
        match self {
            Type::Void => "void",
            Type::Int => "int"
        }
    }
}

impl ExecutionSpaceSpecifier {
    fn to_str(&self) -> &str {
        match self {
            ExecutionSpaceSpecifier::GlobalSpecifier => "__global__"
        }
    }
}

impl MemorySpaceSpecifier {
    fn to_str(&self) -> &str {
        match self {
            MemorySpaceSpecifier::DeviceSpecifier => "__device__"
        }
    }
}

impl TranslationUnitDecl {
    fn to_tokens(&self) -> c::Tokens {
        quote! {
            $(for d in &self.0 join ($['\r']) => $(d.to_tokens()))
        }
    }
}

impl Decl {
    fn to_tokens(&self) -> c::Tokens {
        match self {
            Decl::FunctionDecl(f) => f.to_tokens(),
            Decl::VarDecl(v) => v.to_tokens()
        }
    }
}

impl FunctionDecl {
    fn to_tokens(&self) -> c::Tokens {
        quote!{
            $(self.space.to_str()) $(self.return_type.to_str()) $(&self.name)() $(self.statement.to_tokens())
        }
    }
}

impl VarDecl {
    fn to_tokens(&self) -> c::Tokens {
        quote!{
            $(self.space.to_str()) $(self.val_type.to_str()) $(&self.name);
        }
    }
}

impl Statement {
    fn to_tokens(&self) -> c::Tokens {
        match self {
            Statement::AssignStatement(a) => a.to_tokens(),
            Statement::CompoundStatement(s) =>  s.to_tokens(),
            Statement::IfStatement(i) => i.to_tokens()
        }
    }
}

impl AssignStatement {
    fn to_tokens(&self) -> c::Tokens {
        quote!(
            $(&self.name) = $(self.value.to_tokens());
        )
    }
}

impl CompoundStatement {
    fn to_tokens(&self) -> c::Tokens {
        quote!(
            {
                  $(for statement in &self.0 => $(statement.to_tokens()))
            }
        )
    }
}

impl IfStatement {
    fn to_tokens(&self) -> c::Tokens {
        quote!(
            if ($(self.cond.to_tokens())) $(self.then.to_tokens())
            $(if let Some(else_branch) = &self._else {
                else $(else_branch.to_tokens())
            }))
    }
}

impl Expression {
    fn to_tokens(&self) -> c::Tokens {
        match self {
            Expression::CUDABuiltin(c) => c.to_tokens(),
            Expression::IntegerLiteral(i) => quote!($(*i)),
            Expression::Comparison { operator, left, right } => quote!($(left.to_tokens()) $(operator.to_tokens()) $(right.to_tokens()))
        }
    }
}

impl CUDABuiltin {
    fn to_tokens(&self) -> c::Tokens {
        match &self {
            CUDABuiltin::ThreadIdx(dim) => quote!(threadIdx.$(dim.to_tokens()))
        }
    }
}

impl Dimension {
    fn to_tokens(&self) -> c::Tokens {
        match self {
            Dimension::X => quote!(x),
            Dimension::Y => quote!(y),
            Dimension::Z => quote!(z),
        }
    }
}

impl ComparisonOperator {
    fn to_tokens(&self) -> c::Tokens {
        match &self { ComparisonOperator::Equal => quote!(==) }
    }
}
#[cfg(test)]
mod tests {
    use crate::cuda::{AssignStatement, CUDABuiltin, ComparisonOperator, CompoundStatement, Decl, Dimension, Expression, FunctionDecl, IfStatement, MemorySpaceSpecifier, Statement, TranslationUnitDecl, Type, VarDecl};
    use crate::cuda::ExecutionSpaceSpecifier::GlobalSpecifier;
    use crate::cuda::Type::Void;

    #[test]
    fn emit_noop_program() {
        let program = TranslationUnitDecl(vec![
            Decl::FunctionDecl(FunctionDecl{
                name: "noop".into(),
                return_type: Void,
                space: GlobalSpecifier,
                statement: Statement::CompoundStatement(CompoundStatement(vec![]))
            }),
        ]);
        let output = program
            .to_tokens()
            .to_file_string()
            .expect("failed to render CUDA");

        let expected = "__global__ void noop() {}";
        assert_eq!(output.trim_end(), expected.trim_end());
    }

    #[test]
    fn emit_conditional_program() {
        let program = TranslationUnitDecl(vec![
            Decl::VarDecl(VarDecl{
                name: "g".into(),
                val_type: Type::Int,
                space: MemorySpaceSpecifier::DeviceSpecifier,
            }),
            Decl::FunctionDecl(FunctionDecl{
                name: "conditional".into(),
                return_type: Void,
                space: GlobalSpecifier,
                statement: Statement::CompoundStatement(CompoundStatement(vec![
                    Statement::IfStatement(IfStatement{
                        cond: Expression::Comparison {
                            operator: ComparisonOperator::Equal,
                            left: Box::new(Expression::CUDABuiltin(CUDABuiltin::ThreadIdx(Dimension::X))),
                            right: Box::new(Expression::IntegerLiteral(0)),
                        },
                        then: Box::from(Statement::AssignStatement(AssignStatement{
                            name: "g".into(),
                            value: Expression::IntegerLiteral(1)
                        })),
                        _else: None
                    }),
                ])),
            })
        ]);
        let output = program
            .to_tokens()
            .to_file_string()
            .expect("failed to render CUDA");

        let expected = concat!(
            "__device__ int g;\n",
            "__global__ void conditional() {\n",
            "    if (threadIdx.x == 0) g = 1;\n",
            "}",
        );
        assert_eq!(output.trim_end(), expected.trim_end());
    }
}
