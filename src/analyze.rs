//! analisis on the ast

use crate::{ast, ast::Expr};
use std::collections::HashMap;

/// error for a code analyze
#[derive(Debug)]
pub enum AnalysisError {
    /// set two times same var
    DoubleSet,
    /// A variable does not exist
    VarNotExist(String),
    /// bigger than 2^64 num
    NumberTooBig,
}

/// the level of the variable: static, fn, local
#[derive(Debug, Copy, Clone)]
enum VarLevel {
    /// function scope
    _Func,
    /// static variable scope
    Static,
    /// local scope: if stmts...
    Local,
}

/// a wrapper function to analize the ast
pub fn analize(ast: &mut ast::AstRoot) -> Result<(), AnalysisError> {
    let mut analizer = Analyser::new();
    analizer.analyze(ast, VarLevel::Static)?;
    Ok(())
}
#[derive(Debug)]
struct Analyser {
    /// the initialized_static_vars
    initialized_static_vars: HashMap<String, bool>,
    /// the initialized_local_vars
    initialized_local_vars: HashMap<String, bool>,
}

impl Analyser {
    /// create an Analyser
    pub fn new() -> Self {
        Self {
            initialized_static_vars: HashMap::new(),
            initialized_local_vars: HashMap::new(),
        }
    }

    /// analize a tree to see if works
    pub fn analyze(
        self: &mut Self,
        tree: &mut ast::AstRoot,
        level: VarLevel,
    ) -> Result<HashMap<String, bool>, AnalysisError> {
        let mut new_locals: HashMap<String, bool> = HashMap::new();
        for node in tree.tree.iter_mut() {
            match node {
                ast::AstNode::SetOrChange {
                    sete,
                    setor,
                    change,
                } => {
                    if !*change {
                        if !self.initialized_static_vars.contains_key(sete)
                            && !self.initialized_local_vars.contains_key(sete)
                        {
                            match level {
                                VarLevel::Static => {
                                    self.initialized_static_vars.insert(sete.clone(), true);
                                }
                                VarLevel::Local => {
                                    self.initialized_local_vars.insert(sete.clone(), true);
                                    new_locals.insert(sete.clone(), true);
                                }
                                _ => unimplemented!(),
                            }
                        } else {
                            return Err(AnalysisError::DoubleSet);
                        }
                    } else {
                        self.make_sure_var_exists(sete.clone(), level)?;
                        self.check_expr(setor.clone(), level)?;
                    }
                }
                ast::AstNode::If {
                    guard,
                    body,
                    vars_declared,
                } => {
                    self.check_expr(guard.clone(), level)?;
                    *vars_declared = Some(self.analyze(body, VarLevel::Local)?);
                }
            }
        }
        // drop all the local vars.
        for (key, _) in new_locals.iter() {
            self.initialized_local_vars.remove(key);
        }
        Ok(new_locals)
    }
    fn make_sure_var_exists(
        self: &Self,
        var: String,
        level: VarLevel,
    ) -> Result<(), AnalysisError> {
        match level {
            VarLevel::Static => {
                if !self.initialized_static_vars.contains_key(&var) {
                    return Err(AnalysisError::VarNotExist(var));
                }
            }
            VarLevel::Local => {
                // TODO change for functions. prolly needs a whole redoing
                if !(self.initialized_local_vars.contains_key(&var)
                    || self.initialized_static_vars.contains_key(&var))
                {
                    return Err(AnalysisError::VarNotExist(var));
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
    fn check_expr(self: &Self, expr: Expr, level: VarLevel) -> Result<(), AnalysisError> {
        match expr {
            Expr::Number(n) => check_num(n)?,
            Expr::Iden(s) => self.make_sure_var_exists(s, level)?,
            Expr::BinOp { lhs, op: _, rhs } => {
                self.check_expr(*lhs, level)?;
                self.check_expr(*rhs, level)?;
            }
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
    fn analyze_good_analyze() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to (10+4). set y to (5+x) . change  x to 445235+y .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_double_set() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set x to 5 . change  x to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_change_w_o_set() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . change  z to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    fn analyze_if_scope() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. if x > 10, set z to 4. change  z to 445235 .! set z to 4.";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_bad_if_scope() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10.if x > 10, set z to 4.change  z to 445235.! change z to 4.";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_number_too_big() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        // number is too big
        let input = "Set x to 10. set y to 5 . change  x to 11111111111144523111111111115 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let mut ast = parser.parse(true).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
}
