//! the module where the abstract syntax tree is defined

/// The tree of `set Iden to Expr`
#[derive(Debug)]
pub struct SetNode {
    /// the thing that is being set. an identifier
    pub sete: String,
    /// the thing that the sete is being set to
    pub setor: Expr,
}

/// an expression
#[derive(Debug)]
pub enum Expr {
    /// a number
    Number(String),
}

/// all the types of an ast node. it is like a tagged union. it also holds the values of the ast node type
#[derive(Debug)]
pub enum AstNodeType {
    /// set an expression value to an identifier
    Set(SetNode),
}

/// an abstract syntax tree
#[derive(Debug)]
pub struct Ast {
    /// the nodes in the tree
    pub nodes: Vec<AstNode>,
}

/// the ast node itself
#[derive(Debug)]
pub struct AstNode {
    // /// the row of the node
    // pub row: u32,
    // /// the col of the node
    // pub col: u32,
    /// the actual node
    pub node_type: AstNodeType,
}
