//! the module where the abstract syntax tree is defined. we dont need tests in this because very litle code

use crate::lexer::Token;

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
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add,
    Sub,
}

/// all the types of an ast node. it is like a tagged union. it also holds the values of the ast node type
#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    /// set an expression value to an identifier
    // TODO make it so that each one is just an {} enum so that no need for extra structs
    SetOrChange {
        sete: String,
        setor: Expr,
        change: bool,
    },
}

/// an abstract syntax tree
#[derive(Debug, PartialEq, Clone)]
pub struct Ast {
    /// the nodes in the tree
    pub nodes: Vec<AstNode>,
}
impl Ast {
    /// create an ast
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}
