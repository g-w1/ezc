//! analisis on the ast
use crate::{ast, ast::Expr};
use std::collections::HashMap;

/// error for a code analyze
#[derive(Debug)]
pub enum AnalysisError {
    /// set two times same var
    DoubleSet,
    /// change a variable that is not set
    ChangeUsedBeforeSet,
    /// bigger than 2^64 num
    NumberTooBig,
}

/// the state of an analisis
#[derive(Debug)]
pub struct AnalyzeState {
    initialized_vars: HashMap<String, bool>,
}

impl AnalyzeState {
    /// create an AnalyzeState
    pub fn new() -> AnalyzeState {
        AnalyzeState {
            initialized_vars: HashMap::new(),
        }
    }
    /// analize a tree to see if works
    pub fn analyze(self: &mut Self, tree: &ast::Ast) -> Result<(), AnalysisError> {
        for node in tree.nodes.clone() {
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
            Expr::Number(n) => check_num(n)?,
        }
        Ok(())
    }
}

/// check if a num literal is > 64 bit
fn check_num(num: String) -> Result<(), AnalysisError> {
    match num.parse::<u64>() {
        Ok(_) => Ok(()),
        Err(_) => Err(AnalysisError::NumberTooBig),
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_good_analyze() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . change  x to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut analizer = analyze::AnalyzeState::new();
        analizer.analyze(&ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_double_set() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set x to 5 . change  x to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut analizer = analyze::AnalyzeState::new();
        analizer.analyze(&ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_change_w_o_set() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . change  z to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut analizer = analyze::AnalyzeState::new();
        analizer.analyze(&ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_number_too_big() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        // number is too big
        let input = "Set x to 10. set y to 5 . change  x to 11111111111144523111111111115 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut analizer = analyze::AnalyzeState::new();
        analizer.analyze(&ast).unwrap();
    }
}
