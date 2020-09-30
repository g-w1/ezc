//! analisis on the ast

use crate::{ast, ast::AstNode, ast::Expr};
use std::collections::HashMap;
use std::collections::HashSet;

/// error for a code analyze
#[derive(Debug)]
pub enum AnalysisError {
    /// set two times same var
    DoubleSet(String),
    /// A variable does not exist
    VarNotExist(String),
    /// bigger than 2^64 num
    NumberTooBig(String),
    /// try to set a var in loop. doesn't work for technical reasons
    SetInLoop,
    /// a break without a loop
    BreakWithoutLoop,
    /// Return outside of func
    ReturnOutSideOfFunc,
    /// the function already exists
    FuncAlreadyExists(String),
    /// same arg for function
    SameArgForFunction(String),
}

/// a way to see what ur in
#[derive(Debug, Copy, Clone)]
struct Scope {
    in_if: bool,
    in_loop: bool,
    in_func: bool,
}

/// a wrapper function to analize the ast
pub fn analize(ast: &mut ast::AstRoot) -> Result<(), AnalysisError> {
    let mut analizer = Analyser::new();
    analizer.analyze(&mut ast.tree)?;
    ast.static_vars = Some(get_all_var_decls(&ast.tree));
    Ok(())
}
#[derive(Debug)]
struct Analyser {
    /// the initialized_static_vars
    initialized_static_vars: HashSet<String>,
    /// the initialized_local_vars
    initialized_local_vars: HashMap<String, u32>,
    /// the initialized_function_names
    initialized_function_names: HashSet<String>,
    initialized_function_vars: HashMap<String, u32>,
    /// scope that the analizer is in rn
    scope: Scope,
}

impl Analyser {
    /// create an Analyser
    pub fn new() -> Self {
        Self {
            initialized_static_vars: HashSet::new(),
            initialized_local_vars: HashMap::new(),
            initialized_function_names: HashSet::new(),
            initialized_function_vars: HashMap::new(),
            scope: Scope {
                in_func: false,
                in_if: false,
                in_loop: false,
            },
        }
    }

    /// analize a tree to see if works
    pub fn analyze(
        self: &mut Self,
        tree: &mut Vec<ast::AstNode>,
    ) -> Result<HashMap<String, u32>, AnalysisError> {
        let mut new_locals: HashMap<String, u32> = HashMap::new();
        let mut num_vars_declared: u32 = 0;
        for node in tree.iter_mut() {
            match node {
                ast::AstNode::SetOrChange {
                    sete,
                    setor,
                    change,
                } => {
                    if !*change {
                        if self.scope.in_loop {
                            return Err(AnalysisError::SetInLoop);
                        }
                        if !self.scope.in_func {
                            if !self.initialized_static_vars.contains(sete)
                                && !self.initialized_local_vars.contains_key(sete)
                            {
                                self.check_expr(setor)?;
                                match self.scope {
                                    // Scope::InNone => {
                                    Scope {
                                        in_loop: false,
                                        in_func: false,
                                        in_if: false,
                                    } => {
                                        self.initialized_static_vars.insert(sete.to_owned());
                                    }
                                    // Scope::InLoopAndIf | Scope::InIf => {
                                    Scope {
                                        in_if: true,
                                        in_loop: _,
                                        in_func: false,
                                    } => {
                                        num_vars_declared += 1;
                                        self.initialized_local_vars
                                            .insert(sete.to_owned(), num_vars_declared);
                                        new_locals.insert(sete.to_owned(), num_vars_declared);
                                    }
                                    // Scope::InLoop => unreachable!(),
                                    Scope {
                                        in_loop: true,
                                        in_if: _,
                                        in_func: _,
                                    } => return Err(AnalysisError::SetInLoop),
                                    Scope {
                                        in_func: true,
                                        in_if: _,
                                        in_loop: _,
                                    } => unreachable!(),
                                }
                            } else {
                                return Err(AnalysisError::DoubleSet(sete.to_owned()));
                            }
                        } else {
                            ////////// WE must be in function scope
                            if self.scope.in_loop {
                                return Err(AnalysisError::SetInLoop);
                            }
                            if !self.initialized_function_vars.contains_key(sete) {
                                num_vars_declared += 1;
                                self.initialized_function_vars
                                    .insert(sete.clone(), num_vars_declared);
                            } else {
                                return Err(AnalysisError::DoubleSet(sete.clone()));
                            }
                        }
                    } else {
                        self.make_sure_var_exists(sete)?;
                        self.check_expr(setor)?;
                    }
                }
                ast::AstNode::If {
                    guard,
                    body,
                    vars_declared,
                } => {
                    self.check_expr(guard)?;
                    // if let Scope::InLoop | Scope::InLoopAndIf = scope {
                    let tmp_scope = self.scope;
                    match self.scope {
                        Scope {
                            in_if: _,
                            in_loop: t,
                            in_func: false,
                        } => {
                            self.scope = Scope {
                                in_loop: t,
                                in_func: false,
                                in_if: true,
                            };
                            // *vars_declared = Some(self.analyze(body, Scope::InLoopAndIf)?);
                            *vars_declared = Some(self.analyze(body)?);
                            // return scope to what it was after changing it
                            self.scope = tmp_scope;
                        }
                        Scope {
                            in_if: _,
                            in_loop: t,
                            in_func: true,
                        } => {
                            self.scope = Scope {
                                in_loop: t,
                                in_func: true,
                                in_if: true,
                            };
                            // *vars_declared = Some(self.analyze(body, Scope::InIf)?);
                            *vars_declared = Some(self.analyze(body)?);
                            self.scope = tmp_scope;
                        }
                    }
                }
                ast::AstNode::Loop { body } => {
                    let tmp_scope = self.scope;
                    self.scope = Scope {
                        in_loop: true,
                        in_if: false,
                        ..self.scope
                    };
                    // self.analyze(body, Scope::InLoop)?;
                    self.analyze(body)?;
                    self.scope = tmp_scope;
                }
                ast::AstNode::Func {
                    name,
                    args,
                    body,
                    vars_declared,
                } => {
                    // TODO get rid of .clone
                    /////////////// Making sure function name doesn't exist
                    if !self.initialized_function_names.insert(name.clone()) {
                        return Err(AnalysisError::FuncAlreadyExists(name.clone()));
                    }
                    ////////////////////// Making sure there no duplicate args
                    let mut args_map = HashSet::new();
                    for n in args {
                        if !args_map.insert(n.clone()) {
                            return Err(AnalysisError::SameArgForFunction(n.to_owned()));
                        }
                        // TODO it is 0 because its not really allocated its just weird. idk
                        self.initialized_function_vars.insert(n.clone(), 0);
                    }
                    /////////////////// The body
                    let tmp_scope = self.scope;
                    self.scope = Scope {
                        in_if: false,
                        in_func: true,
                        in_loop: false,
                    };
                    *vars_declared = Some(self.analyze(body)?);
                    self.scope = tmp_scope;
                    // clear the function vars since we may wanna do another function
                    self.initialized_function_vars.clear();
                }
                ast::AstNode::Break => {
                    // if let Scope::InLoop | Scope::InLoopAndIf = scope {
                    if let Scope {
                        in_if: _,
                        in_loop: true,
                        in_func: _,
                    } = self.scope
                    {
                    } else {
                        return Err(AnalysisError::BreakWithoutLoop);
                    }
                }
                ast::AstNode::Return { val } => {
                    if self.scope.in_func {
                        self.check_expr(val)?;
                    } else {
                        return Err(AnalysisError::ReturnOutSideOfFunc);
                    }
                }
            }
        }
        // drop all the local vars.
        for (key, _) in new_locals.iter() {
            self.initialized_local_vars.remove(key);
        }
        Ok(new_locals)
    }
    /// a helper function to make sure a variable exists
    fn make_sure_var_exists(&self, var: &String) -> Result<(), AnalysisError> {
        if !self.scope.in_func {
            if !self.initialized_local_vars.contains_key(var)
                && !self.initialized_static_vars.contains(var)
            {
                return Err(AnalysisError::VarNotExist(var.to_owned()));
            }
        } else {
            if !self.initialized_function_vars.contains_key(var) {
                return Err(AnalysisError::VarNotExist(var.to_owned()));
            }
        }
        Ok(())
    }
    /// analyze an expression
    fn check_expr(self: &Self, expr: &Expr) -> Result<(), AnalysisError> {
        match expr {
            Expr::Number(n) => check_num(n)?,
            Expr::Iden(s) => self.make_sure_var_exists(&s)?,
            Expr::BinOp { lhs, op: _, rhs } => {
                self.check_expr(lhs)?;
                self.check_expr(rhs)?;
            }
        }
        Ok(())
    }
}

/// check if a num literal is > 64 bit
fn check_num(num: &String) -> Result<(), AnalysisError> {
    match num.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(AnalysisError::NumberTooBig(num.to_owned())),
    }
}

/// get all the variable declarations in a block.
fn get_all_var_decls(tree: &Vec<AstNode>) -> Vec<String> {
    let mut vars = Vec::new();
    for node in tree {
        match node {
            AstNode::SetOrChange {
                sete,
                setor: _,
                change: false,
            } => {
                vars.push(sete.to_owned());
            }
            _ => {}
        }
    }
    vars
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
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
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
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_use_itself_in_set() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "set y to 0. if (4 + 1) = 5,
            set a to a + 1.
            change y to 5.
        !
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
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
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    fn analyze_if_scope() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. if x > 10, set z to 4. change  z to 445235 .! set z to 4.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_bad_if_scope() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. if x > 10,
                    set z to 4. change  z to 445235.
                !
            change z to 4.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_breaking_bad() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "set z to 4.
            break.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    fn analyze_breaking() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        let input = "loop, break.!";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
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
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    fn analyze_functions() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        // number is too big
        let input = "set x to 4.

function test(y,z,b),

    loop,
      change y to y + 1.
      if y > 10,
        break.
        return 6.
      !
    !

    return y.

!

function lol(),
    return 0.
!
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_bad_functions_0() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        // number is too big
        let input = "set x to 4.
function test(y,z,b),

    change x to 6.
    loop,
      change y to y + 1.
      if y > 10,
        break.
      !
    !
    return y.
!

function lol(y),
!
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
    #[test]
    #[should_panic]
    fn analyze_bad_functions_1() {
        use crate::analyze;
        use crate::lexer;
        use crate::parser;
        let mut tokenizer = lexer::Tokenizer::new();
        // number is too big
        let input = "
function test(y,z,b),

    loop,
      if y != 10,
        break.
        loop,
        !
      !
    !
    return y.
!
change y to 4.

function lol(y),
!
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
    }
}
