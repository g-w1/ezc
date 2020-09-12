//! the parser module

use crate::ast::*;
use crate::lexer::{Locs, Token};

/// the parser
#[derive(Debug)]
pub struct Parser {
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
    ExpectedToken(Token, (u32, u32)),
    FoundToken(Token, (u32, u32)),
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
            Token::BoPlus => (10),
            Token::BoMinus => (10),
            _ => (-1),
        }
    }
    /// a wrapper to give the err not have to use stuff in the functions
    fn expected_token_err(self: &Self, token: Token) -> ParserError {
        ParserError::ExpectedToken(
            token,
            (
                self.locs_input[self.pos_input].0,
                self.locs_input[self.pos_input].1,
            ),
        )
    }
    /// error helper function to put the positions of the tokens in the error message
    fn found_token_err(self: &Self, token: Token) -> ParserError {
        ParserError::FoundToken(
            token,
            (
                self.locs_input[self.pos_input].0,
                self.locs_input[self.pos_input].1,
            ),
        )
    }
    // /// Peek one token ahead without eating it. may need in future
    // fn peek(self: &mut Self) -> Token {
    //     self.input[self.pos_input + 1].clone()
    // }
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
        Err(self.found_token_err(token))
    }
    /// The function that does the parsing
    pub fn parse(self: &mut Self) -> Result<Ast, ParserError> {
        let mut tree = Ast::new();
        while self.cur_tok() != Token::Eof {
            match self.cur_tok() {
                Token::Kset => self.parse_set_stmt(&mut tree)?,
                Token::Kchange => self.parse_change_stmt(&mut tree)?,
                Token::Eof => break,
                t => return Err(self.found_token_err(t)),
            }
        }
        Ok(tree)
    }
    /// Iden <- String
    fn parse_iden(self: &mut Self) -> Result<String, ParserError> {
        match self.cur_tok() {
            Token::Iden(s) => {
                self.pos_input += 1;
                Ok(s)
            }
            t => {
                return Err(ParserError::FoundToken(t, (0, 0)));
            }
        }
    }
    //
    // Expression parsing
    //
    /// Iden <- String (this is different than parse_iden because it returns a wrapped expr instead of string so less wrapping is needed at the upper level)
    fn parse_expr_iden(self: &mut Self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::Iden(s) => {
                self.pos_input += 1;
                Ok(Expr::Iden(s))
            }
            t => {
                return Err(self.expected_token_err(t));
            }
        }
    }
    /// Number <- String
    fn parse_expr_number(self: &mut Self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::IntLit(s) => {
                self.pos_input += 1;
                Ok(Expr::Number(s))
            }
            t => Err(self.expected_token_err(t)),
        }
    }
    /// Expr <- Number | Iden | ParenExpr | Expr BinOp Expr
    fn parse_expr(self: &mut Self) -> Result<Expr, ParserError> {
        let lhs = self.parse_expr_primary()?;
        self.parse_bin_op_rhs(0, &lhs)
    }
    /// TODO add docstring
    // impliments this algorithm https://en.wikipedia.org/wiki/Operator-precedence_parser
    fn parse_bin_op_rhs(
        self: &mut Self,
        passed_pres: i8,
        old_lhs: &Expr,
    ) -> Result<Expr, ParserError> {
        let mut pres: i8;
        let mut lhs = old_lhs.clone();
        loop {
            pres = self.bin_op_pres();
            if pres < passed_pres {
                return Ok(lhs);
            }
            // this has to be binop because other things have -1 stuff
            let bin_op = self.cur_tok();
            // eat the bin op. doing let _ to show that it returns something
            let _ = self.next();
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
            t => return Err(self.found_token_err(t)),
        }
        let _  = self.next();
        // parse the expression (yay recursion is fun)
        let parsed_expr = self.parse_expr()?;
        // eat Rparen
        match self.cur_tok() {
            Token::Rparen => {}
            t => return Err(self.found_token_err(t)),
        }
        let _  = self.next();
        Ok(parsed_expr)
    }
    /// Expr <- Number | Iden | ParenExpr | Expr BinOp Expr (parsing an expression but not top level)
    fn parse_expr_primary(self: &mut Self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::IntLit(_) => self.parse_expr_number(),
            Token::Iden(_) => self.parse_expr_iden(),
            Token::Lparen => self.parse_expr_paren(),
            t => return Err(self.expected_token_err(t)),
        }
    }
    //
    // Parsing stmts
    //
    /// SetNode <- Kset KIden Kto Expr EndOfLine
    fn parse_set_stmt(self: &mut Self, tree: &mut Ast) -> Result<(), ParserError> {
        // Kset
        self.expect_eat_token(Token::Kset)?;
        // Iden
        let sete = self.parse_iden()?;
        // Kto
        self.expect_eat_token(Token::Kto)?;
        // Expr
        let setor = self.parse_expr()?;
        let node = SetNode { sete, setor };
        // EndOfLine
        self.expect_eat_token(Token::EndOfLine)?;
        tree.nodes.push(AstNode::Set(node));
        Ok(())
    }
    /// ChangeNode <- Kchange KIden Kto Expr EndOfLine
    fn parse_change_stmt(self: &mut Self, tree: &mut Ast) -> Result<(), ParserError> {
        // Kset
        self.expect_eat_token(Token::Kchange)?;
        // Iden
        let sete = self.parse_iden()?;
        // Kto
        self.expect_eat_token(Token::Kto)?;
        // Expr
        let setor = self.parse_expr()?;
        let node = ChangeNode { sete, setor };
        // EndOfLine
        self.expect_eat_token(Token::EndOfLine)?;
        tree.nodes.push(AstNode::Change(node));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Ast, AstNode, Expr, SetNode};
    use crate::lexer;
    #[test]
    fn parser_set_expr() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(String::from(
            "Set x to 10. set y to 5 . set  xarst to 555134234523452345  \n.\n\n",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        assert_eq!(
            Ast {
                nodes: vec![
                    AstNode::Set(SetNode {
                        sete: String::from("x"),
                        setor: Expr::Number(String::from("10"))
                    }),
                    AstNode::Set(SetNode {
                        sete: String::from("y"),
                        setor: Expr::Number(String::from("5"))
                    }),
                    AstNode::Set(SetNode {
                        sete: String::from("xarst"),
                        setor: Expr::Number(String::from("555134234523452345"))
                    })
                ]
            },
            ast
        );
    }
    #[test]
    #[should_panic]
    fn parser_bad_stuff() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(String::from(
            "Set x to 10. set y to 5 . set  xarst to 555134234523452345. set 6 to lol.",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        parser.parse().unwrap();
    }
}
