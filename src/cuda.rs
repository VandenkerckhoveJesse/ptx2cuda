pub enum Type {
    Int,
    Void
}

pub struct TranslationUnitDecl (pub Vec<Decl>);

pub enum Decl {
    FunctionDecl(FunctionDecl),
    VarDecl(VarDecl),
}

pub struct FunctionDecl {
    pub name: String,
    pub return_type: Type,
    pub space: ExecutionSpaceSpecifier,
    pub statement: Statement
}

pub struct VarDecl{
    pub name: String,
    pub val_type: Type,
    pub space: MemorySpaceSpecifier
}


pub enum ExecutionSpaceSpecifier {
    GlobalSpecifier
}

pub enum MemorySpaceSpecifier {
    DeviceSpecifier
}

pub enum Statement{
    AssignStatement(AssignStatement),
    CompoundStatement(CompoundStatement),
    IfStatement(IfStatement)
}

pub struct AssignStatement {
    pub(crate) name: String,
    pub(crate) value: Expression,
}

pub struct CompoundStatement(pub Vec<Statement>);

pub struct IfStatement {
    pub(crate) cond: Expression,
    pub(crate) then: Box<Statement>,
    pub(crate) _else: Option<Box<Statement>>
}

pub enum Expression {
    CUDABuiltin(CUDABuiltin),
    IntegerLiteral(i64),

    Comparison {
        operator: ComparisonOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    }
}

pub enum ComparisonOperator {
    Equal
}

pub enum CUDABuiltin {
    ThreadIdx(Dimension)
}

pub enum Dimension {
    X, Y, Z
}