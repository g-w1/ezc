//! the module where the abstract syntax tree is defined. we dont need tests in this because very litle code

use crate::lexer::Token;
use std::collections::HashMap;

/// an expression
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    /// a number
    Number(String),
    /// an iden
    Iden(String),
    /// a binop
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
    },
}

pub fn convert_tok_to_ast_binop(tok: Token) -> BinOp {
    match tok {
        Token::BoPlus => BinOp::Add,
        Token::BoMinus => BinOp::Sub,
        Token::BoG => BinOp::Gt,
        Token::BoL => BinOp::Lt,
        Token::BoLe => BinOp::Lte,
        Token::BoGe => BinOp::Gte,
        Token::BoE => BinOp::Equ,
        Token::BoNe => BinOp::Ne,
        Token::BoAnd => BinOp::And,
        Token::BoOr => BinOp::Or,
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Gt,
    Lt,
    Equ,
    Lte,
    Gte,
    Ne,
    And,
    Or
}

/// all the types of an ast node. it is like a tagged union. it also holds the values of the ast node type
#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    /// set an expression value to an identifier
    SetOrChange {
        sete: String,
        setor: Expr,
        change: bool,
    },
    /// an if statement
    If {
        guard: Expr,
        body: Vec<AstNode>,
        /// for the variables declared inside the if statement
        vars_declared: Option<HashMap<String, u32>>,
    },
    Loop {
        body: Vec<AstNode>,
    },
    Break,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AstRoot {
    pub static_vars: Option<Vec<String>>,
    pub tree: Vec<AstNode>,
}
