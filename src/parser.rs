//! the parser module

use crate::lexer::{Token, TokenStream};

/// the parser
pub struct Parser {
    /// the input to the parser as a TokenStream
    input: TokenStream,
    /// ptr to pos in input
    pos_input: usize,
}

/// an error in the parsing
pub enum ParserError {
    ExpectedToken(Token, (u32, u32)),
}

impl Parser {
    pub fn new(input: TokenStream) -> Self {
        Parser {
            input,
            pos_input: 0,
        }
    }
    fn peek_num(self: &Self, i: usize) -> Token {
        self.input[self.pos_input + i].0.clone()
    }
    fn expect_eat_token(
        self: &mut Self,
        token: Token,
        coords: (u32, u32),
    ) -> Result<(), ParserError> {
        if self.peek_num(1) == token {
            self.pos_input += 1;
            return Ok(());
        }
        Err(ParserError::ExpectedToken(token, coords))
    }
}
