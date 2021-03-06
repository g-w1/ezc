//! the parser module

use crate::ast::*;
use crate::lexer::{Locs, Token};
use std::cmp::Ordering;

/// AstRoot <- Vec<Ast>
pub fn parse(input: Vec<Token>, locs_input: Vec<u32>) -> Result<AstRoot, ParserError> {
    let mut tree = Parser::new(input, locs_input).parse(true)?;
    // sort it so that funcs are on top of vec so that codegen is MUCH easier
    tree.sort_by(|a, b| match (a, b) {
        (AstNode::Func { .. }, AstNode::Func { .. }) => Ordering::Equal,
        (AstNode::Extern { .. }, AstNode::Extern { .. }) => Ordering::Equal,
        (AstNode::Extern { .. }, AstNode::Func { .. }) => Ordering::Less,
        (AstNode::Func { .. }, AstNode::Extern { .. }) => Ordering::Greater,
        (AstNode::Func { .. }, _) => Ordering::Less,
        (AstNode::Extern { .. }, _) => Ordering::Less,
        (_, AstNode::Extern { .. }) => Ordering::Greater,
        (_, AstNode::Func { .. }) => Ordering::Greater,
        _ => Ordering::Equal,
    });
    Ok(AstRoot {
        static_vars: None,
        tree,
    })
}

/// the parser
#[derive(Debug)]
struct Parser {
    /// the input to the parser as a Vec<Token>
    input: Vec<Token>,
    /// ptr to pos in input
    pos_input: usize,
    /// the debug info of the locations of the tokens
    locs_input: Locs,
}

/// an error in the parsing
#[derive(Debug)]
pub enum ParserError {
    ExectedOneFoundAnother {
        expected: Token,
        found: Token,
        pos: u32,
    },
}

impl Parser {
    //
    // Helper Functions
    //
    /// create a new parser
    pub fn new(input: Vec<Token>, locs_input: Locs) -> Self {
        Parser {
            input,
            pos_input: 0,
            locs_input,
        }
    }

    /// the presedence of a binary operation
    fn bin_op_pres(self: &Self) -> i8 {
        match self.cur_tok() {
            Token::BoPlus => 10,
            Token::BoMinus => 10,
            Token::BoMul => 10,
            Token::BoG => 5,
            Token::BoL => 5,
            Token::BoLe => 5,
            Token::BoGe => 5,
            Token::BoE => 5,
            Token::BoNe => 5,
            Token::BoOr => 5,
            Token::BoAnd => 5,
            _ => -1,
        }
    }
    /// a wrapper to give the err not have to use stuff in the functions
    fn expected_token_err(self: &Self, expected: Token, found: Token) -> ParserError {
        ParserError::ExectedOneFoundAnother {
            expected,
            found,
            pos: self.locs_input[self.pos_input],
        }
    }
    // /// Peek one token ahead without eating it. may need in future
    fn peek(self: &mut Self) -> Token {
        self.input[self.pos_input + 1].clone()
    }
    /// Get the current token in the stream
    fn cur_tok(self: &Self) -> Token {
        self.input[self.pos_input].clone()
    }
    /// Get the next token and inc self.pos_input
    fn next(self: &mut Self) -> Token {
        self.pos_input += 1;
        self.input[self.pos_input].clone()
    }
    /// A helper function to eat a token only if it exists and return error otherwise
    fn expect_eat_token(self: &mut Self, token: Token) -> Result<(), ParserError> {
        if self.cur_tok() == token {
            self.pos_input += 1;
            return Ok(());
        }
        Err(self.expected_token_err(token, self.cur_tok()))
    }
    /// The function that does the parsing
    fn parse(self: &mut Self, toplevel: bool) -> Result<Vec<AstNode>, ParserError> {
        let mut tree = Vec::new();
        while self.cur_tok() != Token::Eof {
            match self.cur_tok() {
                Token::Kset => self.parse_set_stmt(&mut tree)?,
                Token::Kchange => self.parse_change_stmt(&mut tree)?,
                Token::Kloop => self.parse_loop_stmt(&mut tree)?,
                Token::Kif => self.parse_if_stmt(&mut tree)?,
                Token::ExclaimMark if !toplevel => break,
                Token::Kfunc if toplevel => self.parse_func(&mut tree)?,
                Token::Kexport if toplevel => self.parse_exported_func(&mut tree)?,
                Token::Kextern if toplevel => self.parse_extern(&mut tree)?,
                Token::Kreturn if !toplevel => {
                    self.expect_eat_token(Token::Kreturn)?;
                    let e = self.parse_expr()?;
                    self.expect_eat_token(Token::EndOfLine)?;
                    tree.push(AstNode::Return { val: e });
                }
                Token::Kbreak => {
                    self.expect_eat_token(Token::Kbreak)?;
                    self.expect_eat_token(Token::EndOfLine)?;
                    tree.push(AstNode::Break);
                }
                Token::Eof => break,
                t => return Err(self.expected_token_err(Token::Eof, t)),
            }
        }
        Ok(tree)
    }
    /// Iden <- String
    fn parse_iden(&mut self) -> Result<String, ParserError> {
        match self.cur_tok() {
            Token::Iden(s) => {
                self.pos_input += 1;
                Ok(s)
            }
            t => Err(self.expected_token_err(Token::Iden(String::from("")), t)),
        }
    }
    //
    // Expression parsing
    //
    /// Iden <- String (this is different than parse_iden because it returns a wrapped expr instead of string so less wrapping is needed at the upper level)
    fn parse_expr_iden(&mut self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::Iden(s) => {
                self.pos_input += 1;
                Ok(Expr::Iden(s))
            }
            t => Err(self.expected_token_err(Token::Iden(String::from("")), t)),
        }
    }
    /// Number <- String
    fn parse_expr_number(&mut self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::IntLit(s) => {
                self.pos_input += 1;
                Ok(Expr::Number(s))
            }
            t => Err(self.expected_token_err(Token::IntLit(String::from("")), t)),
        }
    }
    /// Expr <- Number | Iden | ParenExpr | Expr BinOp Expr
    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let lhs = self.parse_expr_primary()?;
        self.parse_bin_op_rhs(0, &lhs)
    }
    /// Expr <- LParen Expr Rparen.
    // impliments this algorithm https://en.wikipedia.org/wiki/Operator-precedence_parser
    fn parse_bin_op_rhs(&mut self, passed_pres: i8, old_lhs: &Expr) -> Result<Expr, ParserError> {
        let mut pres: i8;
        let mut lhs = old_lhs.clone();
        loop {
            pres = self.bin_op_pres();
            if pres < passed_pres {
                return Ok(lhs);
            }
            // this has to be binop because other things have -1 stuff
            let bin_op = self.cur_tok();
            self.next();
            let mut rhs = self.parse_expr_primary()?;
            let next_pres = self.bin_op_pres();
            if pres < next_pres {
                rhs = self.parse_bin_op_rhs(pres + 1, &rhs)?;
                // then loop around
            }
            lhs = Expr::BinOp {
                op: convert_tok_to_ast_binop(bin_op),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
    }
    /// ParenExpr <- Lparen Expr Rparen
    fn parse_expr_paren(self: &mut Self) -> Result<Expr, ParserError> {
        // eat Lparen
        match self.cur_tok() {
            Token::Lparen => {}
            t => return Err(self.expected_token_err(Token::Lparen, t)),
        }
        self.next();
        // parse the expression (yay recursion is fun)
        let parsed_expr = self.parse_expr()?;
        // eat Rparen
        match self.cur_tok() {
            Token::Rparen => {}
            t => {
                return Err(self.expected_token_err(Token::Rparen, t));
            }
        }
        self.next();
        Ok(parsed_expr)
    }
    /// Extern <- Kextern Fnproto.
    fn parse_extern(&mut self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        self.expect_eat_token(Token::Kextern)?;
        self.expect_eat_token(Token::Kfunc)?;
        let (func_name, items_in_func) = self.parse_func_proto()?;
        tree.push(AstNode::Extern {
            name: func_name,
            args: items_in_func,
        });
        self.expect_eat_token(Token::EndOfLine)?;
        Ok(())
    }
    /// Type <- Iden
    /// Types are overrated.
    fn parse_type(&mut self) -> Result<Type, ParserError> {
        // if self.peek() == Token::OpenBrak {
        //     self.expect_eat_token(Token::OpenBrak)?;
        //     let num_of_items_in_arr: String;
        //     if let Token::IntLit(x) = self.next() {
        //         num_of_items_in_arr = x;
        //     } else {
        //         return Err(
        //             self.expected_token_err(Token::IntLit(String::from("0")), self.cur_tok())
        //         );
        //     }
        //     self.expect_eat_token(Token::CloseBrak)?;
        //     Ok(Type::ArrNum(self.parse_iden()?, num_of_items_in_arr))
        // } else {
        Ok(Type::Num(self.parse_iden()?))
        // }
    }
    /// FnProto <- Iden Lparen (Iden ,)* Rparen
    fn parse_func_proto(&mut self) -> Result<(String, Vec<Type>), ParserError> {
        let func_name = self.parse_iden()?;
        let mut items_in_func = Vec::new();
        self.expect_eat_token(Token::Lparen)?;
        if self.cur_tok() == Token::Rparen {
            self.expect_eat_token(Token::Rparen)?;
            return Ok((func_name, items_in_func));
        }
        while let Token::Iden(_) = self.cur_tok() {
            items_in_func.push(self.parse_type()?);
            match self.cur_tok() {
                Token::Comma => self.expect_eat_token(Token::Comma)?,
                Token::Rparen => {
                    self.expect_eat_token(Token::Rparen)?;
                    break;
                }
                t => return Err(self.expected_token_err(Token::Rparen, t)),
            }
        }
        Ok((func_name, items_in_func))
    }
    /// Expr <- Number | Iden | ParenExpr | Expr BinOp Expr (parsing an expression but not top level) | AtSign Iden | Iden OpenBrak Expr CloseBrak
    fn parse_expr_primary(&mut self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::IntLit(_) => self.parse_expr_number(),
            Token::Iden(_) if self.peek() == Token::Lparen => self.parse_expr_funcall(),
            Token::Iden(a) if self.peek() == Token::OpenBrak => {
                // eat the iden
                self.next();
                // i am superstious
                self.expect_eat_token(Token::OpenBrak)?;
                let r = Expr::AccessArray(a, Box::new(self.parse_expr()?));
                self.expect_eat_token(Token::CloseBrak)?;
                Ok(r)
            }
            Token::AtSign => {
                self.next();
                let i = self.parse_iden()?;
                Ok(Expr::DerefPtr(i))
            }
            Token::Iden(_) => self.parse_expr_iden(),
            Token::Lparen => self.parse_expr_paren(),
            t => {
                return Err(self.expected_token_err(Token::Lparen, t));
            }
        }
    }
    /// Expr <- Iden Lparen ParenExpr,* Rparen
    fn parse_expr_funcall(&mut self) -> Result<Expr, ParserError> {
        let func_name = self.parse_iden()?;
        self.expect_eat_token(Token::Lparen)?;
        let mut args = Vec::new();
        if self.cur_tok() == Token::Rparen {
            self.expect_eat_token(Token::Rparen)?;
            return Ok(Expr::FuncCall {
                func_name,
                args,
                external: None,
            });
        }
        while let Token::Iden(_) | Token::IntLit(_) | Token::AtSign = self.cur_tok() {
            args.push(self.parse_val()?);
            match self.cur_tok() {
                Token::Comma => self.expect_eat_token(Token::Comma)?,
                Token::Rparen => {
                    self.expect_eat_token(Token::Rparen)?;
                    break;
                }
                t => return Err(self.expected_token_err(Token::Rparen, t)),
            }
        }
        Ok(Expr::FuncCall {
            func_name,
            args,
            external: None,
        })
    }
    /// Setor <- Expr | ArrLit
    fn parse_val(&mut self) -> Result<Val, ParserError> {
        Ok(match self.cur_tok() {
            Token::OpenBrak => Val::Array(self.parse_arr_lit()?),
            _ => Val::Expr(self.parse_expr()?),
        })
    }
    /// ArrLit <- OpenBrak (Expr Comma)* CloseBrak
    fn parse_arr_lit(&mut self) -> Result<Vec<Expr>, ParserError> {
        self.expect_eat_token(Token::OpenBrak)?;
        let mut items = Vec::new();
        while self.cur_tok() != Token::CloseBrak {
            items.push(self.parse_expr()?);
            if Token::CloseBrak == self.cur_tok() {
                self.expect_eat_token(Token::CloseBrak)?;
                return Ok(items);
            }
            self.next();
        }
        self.expect_eat_token(Token::CloseBrak)?;
        Ok(items)
    }
    //
    // Parsing stmts
    //
    /// Function <- FnProto OpenBlock Ast CloseBlock
    fn parse_func(&mut self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        self.expect_eat_token(Token::Kfunc)?;
        let (name, args) = self.parse_func_proto()?;
        self.expect_eat_token(Token::Comma)?;
        let body = self.parse(false)?;
        self.expect_eat_token(Token::ExclaimMark)?;
        tree.push(AstNode::Func {
            name,
            args,
            body,
            export: false,
            vars_declared: None,
        });
        Ok(())
    }
    /// ExportedFunc <- Kexport Function
    fn parse_exported_func(&mut self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        self.expect_eat_token(Token::Kexport)?;
        self.expect_eat_token(Token::Kfunc)?;
        let (name, args) = self.parse_func_proto()?;
        self.expect_eat_token(Token::Comma)?;
        let body = self.parse(false)?;
        self.expect_eat_token(Token::ExclaimMark)?;
        tree.push(AstNode::Func {
            name,
            args,
            body,
            export: true,
            vars_declared: None,
        });
        Ok(())
    }
    /// LoopNode <- Kloop OpenBlock Ast CloseBlock
    fn parse_loop_stmt(self: &mut Self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        self.expect_eat_token(Token::Kloop)?;
        self.expect_eat_token(Token::Comma)?;
        let body: Vec<AstNode> = self.parse(false)?;
        self.expect_eat_token(Token::ExclaimMark)?;
        tree.push(AstNode::Loop { body });
        Ok(())
    }
    /// IfNode <- Kif Expr OpenBlock Ast CloseBlock
    fn parse_if_stmt(self: &mut Self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        // Kif
        self.expect_eat_token(Token::Kif)?;
        // Expr
        let guard: Expr = self.parse_expr()?;
        // OpenBlock
        self.expect_eat_token(Token::Comma)?;
        // Ast
        let body: Vec<AstNode> = self.parse(false)?;
        // CloseBlock
        self.expect_eat_token(Token::ExclaimMark)?;
        tree.push(AstNode::If {
            guard,
            body,
            vars_declared: None,
        });
        Ok(())
    }
    /// SetNode <- Kset KIden Kto Expr EndOfLine
    fn parse_set_stmt(self: &mut Self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        // Kset
        self.expect_eat_token(Token::Kset)?;
        // Iden
        let sete = self.parse_iden()?;
        self.expect_eat_token(Token::Kto)?;
        // Expr
        let setor = self.parse_val()?;
        let node = AstNode::SetOrChange {
            sete,
            type_of: TypeOfSetOrChange::SetIden,
            setor,
        };
        // EndOfLine
        self.expect_eat_token(Token::EndOfLine)?;
        tree.push(node);
        Ok(())
    }
    /// ChangeNode <- Kchange SpecialSete Kto Expr EndOfLine
    fn parse_change_stmt(self: &mut Self, tree: &mut Vec<AstNode>) -> Result<(), ParserError> {
        // Kset
        self.expect_eat_token(Token::Kchange)?;
        // Iden
        let sete = self.parse_sete_special()?;
        // Kto
        self.expect_eat_token(Token::Kto)?;
        // Expr
        let setor = self.parse_val()?;
        let node = AstNode::SetOrChange {
            sete: sete.0,
            type_of: sete.1,
            setor,
        };
        // EndOfLine
        self.expect_eat_token(Token::EndOfLine)?;
        tree.push(node);
        Ok(())
    }
    /// SpecialSete <- KIden | AtSign KIden | KIden OpenBrak [ Expr ] CloseBlock
    fn parse_sete_special(self: &mut Self) -> Result<(String, TypeOfSetOrChange), ParserError> {
        // using Option<bool> as a crude c element enum
        if let Token::AtSign = self.cur_tok() {
            self.next();
            Ok((self.parse_iden()?, TypeOfSetOrChange::ChangePtrDeref))
        } else {
            let s = self.parse_iden()?;
            if let Token::OpenBrak = self.cur_tok() {
                self.next();
                let e = self.parse_expr()?;
                let r = Ok((s, TypeOfSetOrChange::ChangeArrIndex(e)));
                self.expect_eat_token(Token::CloseBrak)?;
                r
            } else {
                Ok((s, TypeOfSetOrChange::ChangeIden))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Expr, Val};
    use crate::lexer;
    #[test]
    fn parser_set() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "Set x to 10. set y to 5 . set  xarst to 555134234523452345  \n.\n\n",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("10")))
                },
                AstNode::SetOrChange {
                    sete: String::from("y"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("5")))
                },
                AstNode::SetOrChange {
                    sete: String::from("xarst"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("555134234523452345")))
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_funcall() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from("set p to fib(a,b). set z to lib()."));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("p"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::FuncCall {
                        func_name: String::from("fib"),
                        args: vec![
                            Val::Expr(Expr::Iden(String::from("a"))),
                            Val::Expr(Expr::Iden(String::from("b")))
                        ],
                        external: None,
                    })
                },
                AstNode::SetOrChange {
                    sete: String::from("z"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::FuncCall {
                        func_name: String::from("lib"),
                        args: vec![],
                        external: None,
                    })
                },
            ],
            ast
        );
    }
    #[test]
    fn parser_loop() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from("Set x to 1. loop, change x to x+1.!"));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("1")))
                },
                AstNode::Loop {
                    body: vec![AstNode::SetOrChange {
                        sete: String::from("x"),
                        type_of: crate::ast::TypeOfSetOrChange::ChangeIden,
                        setor: Val::Expr(Expr::BinOp {
                            lhs: Box::new(Expr::Iden(String::from("x"))),
                            rhs: Box::new(Expr::Number(String::from("1"))),
                            op: BinOp::Add
                        })
                    }]
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_change() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "Set x to 10. set y to 5 . change  x to y  \n.\n\n",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("10")))
                },
                AstNode::SetOrChange {
                    sete: String::from("y"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("5")))
                },
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::ChangeIden,
                    setor: Val::Expr(Expr::Iden(String::from("y")))
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_arrays() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from("Set x to [1,2,3, PutRust(10)]. "));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![AstNode::SetOrChange {
                sete: String::from("x"),
                type_of: crate::ast::TypeOfSetOrChange::SetIden,
                setor: Val::Array(vec![
                    Expr::Number(String::from("1")),
                    Expr::Number(String::from("2")),
                    Expr::Number(String::from("3")),
                    Expr::FuncCall {
                        func_name: String::from("PutRust"),
                        args: vec![Val::Expr(Expr::Number(String::from("10")))],
                        external: None
                    }
                ])
            },],
            ast
        );
    }
    #[test]
    fn parser_deref() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from("Set x to [1,2]. set z to People(@x)."));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Array(vec![
                        Expr::Number(String::from("1")),
                        Expr::Number(String::from("2")),
                    ])
                },
                AstNode::SetOrChange {
                    sete: String::from("z"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::FuncCall {
                        args: vec![Val::Expr(Expr::DerefPtr(String::from("x"))),],
                        func_name: String::from("People"),
                        external: None
                    })
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_array_access() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from("Set x to [1,2]. set z to x[1]."));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Array(vec![
                        Expr::Number(String::from("1")),
                        Expr::Number(String::from("2")),
                    ])
                },
                AstNode::SetOrChange {
                    sete: String::from("z"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::AccessArray(
                        String::from("x"),
                        Box::new(Expr::Number(String::from("1")))
                    ))
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_function_stmt() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "function test(y,z,b),
                set y to 4.
                return y.
            !
",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![AstNode::Func {
                args: vec![
                    Type::Num(String::from("y")),
                    Type::Num(String::from("z")),
                    Type::Num(String::from("b"))
                ],
                export: false,
                body: vec![
                    AstNode::SetOrChange {
                        sete: String::from("y"),
                        setor: Val::Expr(Expr::Number(String::from("4")),),
                        type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    },
                    AstNode::Return {
                        val: Expr::Iden(String::from("y"))
                    }
                ],
                name: String::from("test"),
                vars_declared: None,
            },],
            ast
        );
    }
    #[test]
    fn parser_function_export() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "export function test(y,z,b),
                set y to 4.
                return y.
            !
",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![AstNode::Func {
                args: vec![
                    Type::Num(String::from("y")),
                    Type::Num(String::from("z")),
                    Type::Num(String::from("b"))
                ],
                export: true,
                body: vec![
                    AstNode::SetOrChange {
                        sete: String::from("y"),
                        setor: Val::Expr(Expr::Number(String::from("4")),),
                        type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    },
                    AstNode::Return {
                        val: Expr::Iden(String::from("y"))
                    }
                ],
                name: String::from("test"),
                vars_declared: None,
            },],
            ast
        );
    }
    #[test]
    fn parser_if_stmt() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "Set x to (5 + 10).set y to 32. if x > 10, if y > 4, change x to 5.!! ",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::BinOp {
                        lhs: Box::new(Expr::Number(String::from("5"))),
                        op: BinOp::Add,
                        rhs: Box::new(Expr::Number(String::from("10")))
                    })
                },
                AstNode::SetOrChange {
                    sete: String::from("y"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::Number(String::from("32")),),
                },
                AstNode::If {
                    guard: Expr::BinOp {
                        lhs: Box::new(Expr::Iden(String::from("x"))),
                        op: BinOp::Gt,
                        rhs: Box::new(Expr::Number(String::from("10")))
                    },
                    body: vec![AstNode::If {
                        guard: Expr::BinOp {
                            lhs: Box::new(Expr::Iden(String::from("y"))),
                            op: BinOp::Gt,
                            rhs: Box::new(Expr::Number(String::from("4")))
                        },
                        body: vec![AstNode::SetOrChange {
                            sete: String::from("x"),
                            type_of: crate::ast::TypeOfSetOrChange::ChangeIden,
                            setor: Val::Expr(Expr::Number(String::from("5")))
                        }],
                        vars_declared: None
                    }],
                    vars_declared: None
                }
            ],
            ast
        );
    }
    #[test]
    fn parser_parens_expr() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "Set x to (5 + 10). change y to (1-x)+x . set  xarst to y+x  \n.\n\n",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        assert_eq!(
            vec![
                AstNode::SetOrChange {
                    sete: String::from("x"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::BinOp {
                        lhs: Box::new(Expr::Number(String::from("5"))),
                        op: BinOp::Add,
                        rhs: Box::new(Expr::Number(String::from("10")))
                    })
                },
                AstNode::SetOrChange {
                    sete: String::from("y"),
                    type_of: crate::ast::TypeOfSetOrChange::ChangeIden,
                    setor: Val::Expr(Expr::BinOp {
                        lhs: Box::new(Expr::BinOp {
                            lhs: Box::new(Expr::Number(String::from("1"))),
                            op: BinOp::Sub,
                            rhs: Box::new(Expr::Iden(String::from("x")))
                        }),
                        op: BinOp::Add,
                        rhs: Box::new(Expr::Iden(String::from("x")))
                    })
                },
                AstNode::SetOrChange {
                    sete: String::from("xarst"),
                    type_of: crate::ast::TypeOfSetOrChange::SetIden,
                    setor: Val::Expr(Expr::BinOp {
                        lhs: Box::new(Expr::Iden(String::from("y"))),
                        op: BinOp::Add,
                        rhs: Box::new(Expr::Iden(String::from("x")))
                    })
                }
            ],
            ast
        );
    }
    #[test]
    #[should_panic]
    fn parser_bad_stuff() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(&String::from(
            "Set x to 10. set y to 5 . set  xarst to 555134234523452345. set 6 to lol.",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        parser.parse(true).unwrap();
    }
}
