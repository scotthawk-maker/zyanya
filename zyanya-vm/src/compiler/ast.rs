/// Abstract Syntax Tree (AST) definition for Zyanya Contract Language (ZCL).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub contract_name: Option<String>,
    pub functions: Vec<FunctionDef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let {
        name: String,
        initializer: Expression,
    },
    Assign {
        name: String,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Return(Option<Expression>),
    Expr(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Number(u64),
    Variable(String),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}
