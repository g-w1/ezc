//! The module responsible for lexing the source code

/// The tokens that get parsed from source
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Keyword Tokens
    /// kword for set
    Kset,
    /// kword for Change
    Kchange,
    /// to
    Kto,
    // Iden tokens
    /// Identifier token
    Iden(String),
    /// IntLit token
    IntLit(String),
    /// EndOfLine token (.)
    EndOfLine,
    /// EndOfFile
    Eof,
    // grouping
    Lparen,
    Rparen,
    // Binops
    BoPlus,
    BoMinus,
}

/// The Error type of a lex
#[derive(Debug)]
pub enum LexError {
    /// char not expected
    UnexpectedChar(char),
}

/// see if a word is an iden or a kword TODO: get this inine test to work
/// ```rust
/// let set = get_kword(String::from("set"));
/// assert!(set == Token::Kset);
/// let random = get_kword(String::from("random"));
/// assert!(set == Token::Iden(String::from("random")));
/// ```
fn get_kword(input: &String) -> Token {
    match input.as_str() {
        "set" => return Token::Kset,
        "Set" => return Token::Kset,
        "change" => return Token::Kchange,
        "Change" => return Token::Kchange,
        "to" => return Token::Kto,
        _ => return Token::Iden(input.to_string()),
    }
}

#[derive(Debug, PartialEq)]
enum LexerState {
    Start,
    InWord,
    InNum,
}

#[derive(Debug, PartialEq)]
/// The thing that does the tokenizing
pub struct Tokenizer {
    /// the state of the lexer
    state: LexerState,
    /// used for storing temporary strings
    intermidiate_string: String,
    // /// the output of the tokenizer
    // pub output: Vec<Token>,
    /// the row
    row: u32,
    /// the colunm
    col: u32,
    /// the position that the tokenizer is at
    pos: usize,
}

/// the type alias for a return type from lexing
pub type Locs = Vec<(u32, u32)>;

impl Tokenizer {
    /// The constructor for a tokenizer
    pub fn new() -> Tokenizer {
        Tokenizer {
            state: LexerState::Start,
            intermidiate_string: String::new(),
            row: 0,
            col: 0,
            pos: 0,
        }
    }
    /// the lex function
    pub fn lex(self: &mut Self, input_string: String) -> (Result<Vec<Token>, LexError>, Locs) {
        let input: Vec<char> = input_string.chars().collect();
        let mut output = Vec::new();
        let mut output_poss: Locs = Vec::new();
        let mut c: char;
        loop {
            if input.len() <= self.pos {
                break;
            } else {
                c = input[self.pos];
            }
            match c {
                '\n' => {
                    self.row += 1;
                    self.col = 0;
                }
                _ => self.col += 1,
            }
            match self.state {
                LexerState::Start => {
                    self.intermidiate_string = String::new();
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' => {
                            self.state = LexerState::InWord;
                            self.intermidiate_string.push(c);
                        }
                        '0'..='9' => {
                            self.state = LexerState::InNum;
                            self.intermidiate_string.push(c);
                        }
                        '.' => {
                            self.end_token(&mut output, &mut output_poss, Token::EndOfLine);
                        }
                        '(' => self.end_token(&mut output, &mut output_poss, Token::Lparen),
                        ')' => self.end_token(&mut output, &mut output_poss, Token::Rparen),
                        '+' => self.end_token(&mut output, &mut output_poss, Token::BoPlus),
                        '-' => self.end_token(&mut output, &mut output_poss, Token::BoMinus),
                        ' ' | '\n' => {}
                        _ => {
                            return (Err(LexError::UnexpectedChar(c)), output_poss);
                        }
                    }
                }
                LexerState::InWord => match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => self.intermidiate_string.push(c),
                    _ => {
                        self.end_token(
                            &mut output,
                            &mut output_poss,
                            get_kword(&self.intermidiate_string),
                        );
                        // put back char
                        self.pos -= 1;
                        self.col -= 1;
                    }
                },
                LexerState::InNum => match c {
                    '0'..='9' => self.intermidiate_string.push(c),
                    _ => {
                        self.end_token(
                            &mut output,
                            &mut output_poss,
                            Token::IntLit(self.intermidiate_string.to_owned()),
                        );
                        // put back char
                        self.pos -= 1;
                        self.col -= 1;
                    }
                },
            }
            self.pos += 1;
        }
        // clean up any loose intermediate strings. should really clean up state and if not start throw an error. but good enough for now. TODO
        match self.intermidiate_string.as_str() {
            "" => {}
            _ => match self.state {
                LexerState::InWord => {
                    self.end_token(
                        &mut output,
                        &mut output_poss,
                        get_kword(&self.intermidiate_string),
                    );
                }
                LexerState::InNum => {
                    self.end_token(
                        &mut output,
                        &mut output_poss,
                        Token::IntLit(self.intermidiate_string.to_owned()),
                    );
                }
                _ => {}
            },
        }
        self.end_token(&mut output, &mut output_poss, Token::Eof);
        (Ok(output), output_poss)
    }
    /// the function to end a token
    fn end_token(
        self: &mut Self,
        output: &mut Vec<Token>,
        output_poss: &mut Locs,
        token_type: Token,
    ) {
        output.push(token_type);
        output_poss.push((self.col, self.row));
        self.intermidiate_string = String::from("");
        self.state = LexerState::Start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexer_one_line() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(String::from("set x to 5."));
        assert!(res.0.is_ok());
        assert_eq!(
            tokenizer,
            Tokenizer {
                state: LexerState::Start,
                intermidiate_string: String::from(""),
                row: 0,
                col: 11,
                pos: 11,
            }
        );
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kset,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::IntLit(String::from("5")),
                Token::EndOfLine,
                Token::Eof,
            ]
        );
        assert_eq!(ts.len(), res.1.len())
    }
    #[test]
    fn lexer_bad_ast() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(String::from("set x to 5. b"));
        assert!(res.0.is_ok());
        assert_eq!(
            tokenizer,
            Tokenizer {
                state: LexerState::Start,
                intermidiate_string: String::from(""),
                row: 0,
                col: 13,
                pos: 13,
            }
        );
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kset,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::IntLit(String::from("5")),
                Token::EndOfLine,
                Token::Iden(String::from("b")),
                Token::Eof,
            ]
        );
        assert_eq!(ts.len(), res.1.len())
    }
    #[test]
    fn lexer_good_input() {
        let mut outputs = Vec::new();
        let bad_inputs = ["set x to 5.", "change y to 10."];
        for i in bad_inputs.iter() {
            let mut tokenizer = Tokenizer::new();
            outputs.push(tokenizer.lex(i.to_string()));
        }
        for i in outputs {
            assert_eq!(i.1.len(), i.0.unwrap().len());
        }
    }
    #[test]
    fn lexer_expr() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(String::from("set x to 5. change x to (5 + x)."));
        assert!(res.0.is_ok());
        // assert_eq!(
        //     tokenizer,
        //     Tokenizer {
        //         state: LexerState::Start,
        //         intermidiate_string: String::from(""),
        //         row: 0,
        //         col: 13,
        //         pos: 13,
        //     }
        // );
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kset,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::IntLit(String::from("5")),
                Token::EndOfLine,
                Token::Kchange,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::Lparen,
                Token::IntLit(String::from("5")),
                Token::BoPlus,
                Token::Iden(String::from("x")),
                Token::Rparen,
                Token::EndOfLine,
                Token::Eof,
            ]
        );
        assert_eq!(ts.len(), res.1.len())
    }
}
