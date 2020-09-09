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
    /// create a new parser
    pub fn new(input: Vec<Token>, locs_input: Locs) -> Self {
        Parser {
            input,
            pos_input: 0,
            locs_input,
        }
    }
    /// a wrapper to give the err not have to use stuff in thejj
    fn expected_token_err(self: &Self, token: Token) -> ParserError {
        ParserError::ExpectedToken(
            token,
            (
                self.locs_input[self.pos_input].0,
                self.locs_input[self.pos_input].1,
            ),
        )
    }
    fn found_token_err(self: &Self, token: Token) -> ParserError {
        ParserError::FoundToken(
            token,
            (
                self.locs_input[self.pos_input].0,
                self.locs_input[self.pos_input].1,
            ),
        )
    }
    /// Peek one token ahead without eating it
    fn peek(self: &mut Self) -> Token {
        self.input[self.pos_input + 1].clone()
    }
    fn cur_tok(self: &mut Self) -> Token {
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
                Token::Eof => break,
                t => return Err(self.found_token_err(t)),
            }
        }
        Ok(tree)
    }
    /// KIden <- String
    fn parse_iden(self: &mut Self) -> Result<String, ParserError> {
        match self.cur_tok() {
            Token::Iden(s) => {
                self.pos_input += 1;
                Ok(s)
            }
            t => {
                println!("{:?}", t);
                return Err(ParserError::FoundToken(t, (0, 0)));
            }
        }
    }
    /// Expr <- IntLit
    fn parse_expr(self: &mut Self) -> Result<Expr, ParserError> {
        match self.cur_tok() {
            Token::IntLit(s) => {
                self.pos_input += 1;
                Ok(Expr::Number(s))
            }
            t => return Err(self.expected_token_err(t)),
        }
    }
    /// SetNode <- Kset KIden Kto Expr EndOfLine
    fn parse_set_stmt(self: &mut Self, tree: &mut Ast) -> Result<(), ParserError> {
        // Kset
        self.expect_eat_token(Token::Kset)?;
        // KIden
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Ast, AstNode, Expr, SetNode};
    use crate::lexer;
    #[test]
    fn test_set_expr() {
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
    fn test_bad_stuff() {
        let mut tokenizer = lexer::Tokenizer::new();
        let output = tokenizer.lex(String::from(
            "Set x to 10. set y to 5 . set  xarst to 555134234523452345. set 6 to lol.",
        ));
        let mut parser = Parser::new(output.0.unwrap(), output.1);
        parser.parse().unwrap();
    }
}
