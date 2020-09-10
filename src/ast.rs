//! the module where the abstract syntax tree is defined

/// The tree of `set Iden to Expr`
#[derive(Debug, PartialEq)]
pub struct SetNode {
    /// the thing that is being set. an identifier
    pub sete: String,
    /// the thing that the sete is being set to
    pub setor: Expr,
}
/// The tree of `change Iden to Expr`
#[derive(Debug, PartialEq)]
pub struct ChangeNode {
    /// the thing that is being set. an identifier
    pub sete: String,
    /// the thing that the sete is being set to
    pub setor: Expr,
}

/// an expression
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// a number
    Number(String),
}

/// all the types of an ast node. it is like a tagged union. it also holds the values of the ast node type
#[derive(Debug, PartialEq)]
pub enum AstNode {
    /// set an expression value to an identifier
    Set(SetNode),
    /// change an expression value to an identifier
    Change(ChangeNode),
}

/// an abstract syntax tree
#[derive(Debug, PartialEq)]
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
