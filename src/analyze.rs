//! analisis on the ast
use crate::{ast, ast::Expr};
use std::collections::HashMap;

/// error for a code analyze
pub enum AnalysisError {
    /// set two times same var
    DoubleSet,
    /// change a variable that is not set
    ChangeUsedBeforeSet,
    /// bigger than 2^64 num
    NumberTooBig,
}

/// the state of an analisis
pub struct AnalyzeState {
    initialized_vars: HashMap<String, bool>,
}

impl AnalyzeState {
    pub fn new() -> AnalyzeState{
        AnalyzeState {
            initialized_vars: HashMap::new()
        }
    }
    /// analize a tree to see if works
    pub fn analyze(self: &mut Self, tree: ast::Ast) -> Result<(), AnalysisError> {
        for node in tree.nodes {
            match node {
                ast::AstNode::Set(s) => {
                    if !self.initialized_vars.contains_key(&s.sete) {
                        self.initialized_vars.insert(s.sete, true);
                    } else {
                        return Err(AnalysisError::DoubleSet);
                    }
                    self.check_expr(s.setor)?;
                }
                ast::AstNode::Change(c) => {
                    if !self.initialized_vars.contains_key(&c.sete) {
                        return Err(AnalysisError::ChangeUsedBeforeSet);
                    }
                    self.check_expr(c.setor)?;
                }
            }
        }
        Ok(())
    }
    fn check_expr(self: &Self, expr: Expr) -> Result<(), AnalysisError> {
        match expr {
            Expr::Number(n) => check_num(n)?
        }
        Ok(())
    }
}

/// check if a num literal is > 64 bit
fn check_num(num: String) ->Result<(), AnalysisError>   {
    match num.parse::<u64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(AnalysisError::NumberTooBig),
    }
}
