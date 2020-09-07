//! the parser module

use crate::ast::*;
use crate::lexer::{Token, TokenStream};

/// the parser
#[derive(Debug)]
pub struct Parser {
    /// the input to the parser as a TokenStream
    input: TokenStream,
    /// ptr to pos in input
    pos_input: usize,
}

/// an error in the parsing
#[derive(Debug)]
pub enum ParserError {
    ExpectedToken(Token, (u32, u32)),
    FoundToken(Token, (u32, u32)),
}

impl Parser {
    /// create a new parser
    pub fn new(input: TokenStream) -> Self {
        Parser {
            input,
            pos_input: 0,
        }
    }
    /// A helper function to eat a token only if it exists and return error otherwise
    fn expect_eat_token(
        self: &mut Self,
        token: Token,
        // coords: (u32, u32),
    ) -> Result<(), ParserError> {
        if self.input[self.pos_input+1].clone().0 == token {
            self.pos_input += 1;
            return Ok(());
        }
        // TODO refactor this. i dont wanna have to propigate the coords into everything. maybe have coords as a method that gives coords of current tok. big TODO. do not allow the ones
        Err(ParserError::FoundToken(token, (1, 1)))
    }
    // TODO refactor return type using type so that I dont have to write this tuple everywhere
    /// The function that does the parsing
    pub fn parse(self: &mut Self) -> Result<Ast, ParserError> {
        let mut tree = Ast { nodes: Vec::new() };
        while self.pos_input < self.input.len() - 1 {
            println!("{:?}", tree);
            println!("{:?}", self.pos_input);
            match self.input[self.pos_input].0.clone() {
                Token::Kset => self.parse_set_stmt(&mut tree)?,
                Token::Eof => break,
                t => return Err(ParserError::FoundToken(t, (2, 2))),
            }
            self.pos_input += 1;
        }
        Ok(tree)
    }
    /// KIden <- String
    fn parse_iden(self: &mut Self) -> Result<String, ParserError> {
        match self.input[self.pos_input+1].clone().0 {
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
    /// Expr <- Number
    fn parse_expr(self: &mut Self) -> Result<Expr, ParserError> {
        match self.input[self.pos_input+1].clone().0 {
            Token::Number(s) => {
                self.pos_input += 1;
                Ok(Expr::Number(s))
            }
            t => return Err(ParserError::ExpectedToken(t, (0, 0))),
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
        tree.nodes.push(AstNode {
            node_type: AstNodeType::Set(node),
        });
        Ok(())
    }
}
