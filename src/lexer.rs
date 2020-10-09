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
    /// If
    Kif,
    /// Loop
    Kloop,
    /// break
    Kbreak,
    /// Function
    Kfunc,
    /// Return
    Kreturn,
    /// External
    Kextern,
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
    /// Left paren
    Lparen,
    /// Right paren
    Rparen,
    /// Open block (',')
    Comma,
    /// Close block ('!')
    ExclaimMark,
    // Binops
    /// '+'
    BoPlus,
    /// '-'
    BoMinus,
    /// '*'
    BoMul,
    /// '>'
    BoG,
    /// '<'
    BoL,
    /// '>='
    BoGe,
    /// '<='
    BoLe,
    /// '=='
    BoE,
    /// '!='
    BoNe,
    /// and
    BoAnd,
    /// or
    BoOr,
}

/// The Error type of a lex
#[derive(Debug)]
pub enum LexError {
    /// char not expected
    UnexpectedChar(char, u32),
}

/// see if a word is an iden or a kword TODO: get this inine test to work
/// ```rust
/// let set = get_kword(String::from("set"));
/// assert!(set == Token::Kset);
/// let random = get_kword(String::from("random"));
/// assert!(set == Token::Iden(String::from("random")));
/// ```
#[inline]
fn get_kword(input: &str) -> Token {
    match input {
        "Set" | "set" => Token::Kset,
        "external" | "External" => Token::Kextern,
        "Change" | "change" => Token::Kchange,
        "to" => Token::Kto,
        "If" | "if" => Token::Kif,
        "Loop" | "loop" => Token::Kloop,
        "Break" | "break" => Token::Kbreak,
        "and" | "And" => Token::BoAnd,
        "or" | "Or" => Token::BoOr,
        "function" | "Function" => Token::Kfunc,
        "return" | "Return" => Token::Kreturn,
        _ => Token::Iden(input.to_string()),
    }
}

#[derive(Debug, PartialEq)]
enum LexerState {
    Start,
    InWord,
    InNum,
    SawLessThan,
    SawEquals,
    SawGreaterThan,
    SawBang,
    InComment,
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
    /// the position that the tokenizer is at
    pos: u32,
}

/// the type alias for a return type from lexing
pub type Locs = Vec<u32>;

impl Tokenizer {
    /// The constructor for a tokenizer
    pub fn new() -> Tokenizer {
        Tokenizer {
            state: LexerState::Start,
            intermidiate_string: String::new(),
            pos: 0,
        }
    }
    /// the lex function
    pub fn lex(self: &mut Self, input_string: &String) -> (Result<Vec<Token>, LexError>, Locs) {
        let input: Vec<char> = input_string.chars().collect();
        let mut output = Vec::new();
        let mut output_poss: Locs = Vec::new();
        let mut c: char;
        loop {
            if input.len() <= self.pos as usize {
                break;
            } else {
                c = input[self.pos as usize];
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
                        '.' => self.end_token(&mut output, &mut output_poss, Token::EndOfLine),
                        ',' => self.end_token(&mut output, &mut output_poss, Token::Comma),
                        '(' => self.end_token(&mut output, &mut output_poss, Token::Lparen),
                        ')' => self.end_token(&mut output, &mut output_poss, Token::Rparen),
                        '+' => self.end_token(&mut output, &mut output_poss, Token::BoPlus),
                        '*' => self.end_token(&mut output, &mut output_poss, Token::BoMul),
                        '-' => self.end_token(&mut output, &mut output_poss, Token::BoMinus),
                        '!' => self.state = LexerState::SawBang,
                        '>' => self.state = LexerState::SawGreaterThan,
                        '<' => self.state = LexerState::SawLessThan,
                        '=' => self.state = LexerState::SawEquals,
                        '[' => self.state = LexerState::InComment,
                        ']' => self.state = LexerState::Start,
                        ' ' | '\n' => {}
                        _ => {
                            return (Err(LexError::UnexpectedChar(c, self.pos)), output_poss);
                        }
                    }
                }
                LexerState::InComment => match c {
                    ']' => self.state = LexerState::Start,
                    _ => {}
                },
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
                    }
                },
                LexerState::SawGreaterThan => match c {
                    '=' => self.end_token(&mut output, &mut output_poss, Token::BoGe),
                    _ => {
                        self.end_token(&mut output, &mut output_poss, Token::BoG);
                        self.pos -= 1;
                    }
                },
                LexerState::SawBang => match c {
                    '=' => {
                        self.end_token(&mut output, &mut output_poss, Token::BoNe);
                    }
                    _ => {
                        self.end_token(&mut output, &mut output_poss, Token::ExclaimMark);
                        self.pos -= 1;
                    }
                },
                LexerState::SawLessThan => match c {
                    '=' => self.end_token(&mut output, &mut output_poss, Token::BoLe),
                    _ => {
                        self.end_token(&mut output, &mut output_poss, Token::BoL);
                        self.pos -= 1;
                    }
                },
                LexerState::SawEquals => {
                    self.end_token(&mut output, &mut output_poss, Token::BoE);
                    // TODO ????????? maybe because i wasted cycle. investigate
                    self.pos -= 1;
                }
            }
            self.pos += 1;
        }
        // clean up state
        match self.state {
            LexerState::SawBang => {
                self.end_token(&mut output, &mut output_poss, Token::ExclaimMark)
            }
            LexerState::InWord => match self.intermidiate_string.as_str() {
                "" => {}
                _ => self.end_token(
                    // TODO i use &mut output and &mut output_poss in every one just make it in self
                    &mut output,
                    &mut output_poss,
                    get_kword(&self.intermidiate_string),
                ),
            },
            LexerState::InNum => {
                self.end_token(
                    &mut output,
                    &mut output_poss,
                    Token::IntLit(self.intermidiate_string.to_owned()),
                );
            }
            LexerState::Start => {}
            LexerState::SawEquals => self.end_token(&mut output, &mut output_poss, Token::BoE),
            LexerState::SawGreaterThan => self.end_token(&mut output, &mut output_poss, Token::BoG),
            LexerState::SawLessThan => self.end_token(&mut output, &mut output_poss, Token::BoL),
            LexerState::InComment => {}
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
        self.intermidiate_string = String::from("");
        output_poss.push(self.pos);
        self.state = LexerState::Start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexer_one_line() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from("set x to 5."));
        assert!(res.0.is_ok());
        assert_eq!(
            tokenizer,
            Tokenizer {
                state: LexerState::Start,
                intermidiate_string: String::from(""),
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
        let res = tokenizer.lex(&String::from("set x to 5. b"));
        assert_eq!(
            tokenizer,
            Tokenizer {
                state: LexerState::Start,
                intermidiate_string: String::from(""),
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
    fn lexer_loop() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from("set x to 4. loop, change x to x + 1.!"));
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kset,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::IntLit(String::from("4")),
                Token::EndOfLine,
                Token::Kloop,
                Token::Comma,
                Token::Kchange,
                Token::Iden(String::from("x")),
                Token::Kto,
                Token::Iden(String::from("x")),
                Token::BoPlus,
                Token::IntLit(String::from("1")),
                Token::EndOfLine,
                Token::ExclaimMark,
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
            outputs.push(tokenizer.lex(&i.to_string()));
        }
        for i in outputs {
            assert_eq!(i.1.len(), i.0.unwrap().len());
        }
    }
    #[test]
    fn lexer_comments() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from(
            "[initalize vars] set x to 5. change x to (5 + x).",
        ));
        assert!(res.0.is_ok());
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
    #[test]
    fn lexer_expr() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from("set x to 5. change x to (5 + x)."));
        assert!(res.0.is_ok());
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
    #[test]
    fn lexer_bangs() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from("if x != 5, break. !"));
        assert!(res.0.is_ok());
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kif,
                Token::Iden(String::from("x")),
                Token::BoNe,
                Token::IntLit(String::from("5")),
                Token::Comma,
                Token::Kbreak,
                Token::EndOfLine,
                Token::ExclaimMark,
                Token::Eof,
            ]
        );
        assert_eq!(ts.len(), res.1.len())
    }
    #[test]
    fn lexer_if_stmt() {
        let mut tokenizer = Tokenizer::new();
        let res = tokenizer.lex(&String::from(
            "
if x >= 5,
    set y to 4.
!
if y =2,
    set z to 4.
!
if test <= 4+2,
    set z to 5.
!
",
        ));
        assert!(res.0.is_ok());
        let ts = res.0.unwrap();
        assert_eq!(
            ts,
            vec![
                Token::Kif,
                Token::Iden(String::from("x")),
                Token::BoGe,
                Token::IntLit(String::from("5")),
                Token::Comma,
                Token::Kset,
                Token::Iden(String::from("y")),
                Token::Kto,
                Token::IntLit(String::from("4")),
                Token::EndOfLine,
                Token::ExclaimMark,
                Token::Kif,
                Token::Iden(String::from("y")),
                Token::BoE,
                Token::IntLit(String::from("2")),
                Token::Comma,
                Token::Kset,
                Token::Iden(String::from("z")),
                Token::Kto,
                Token::IntLit(String::from("4")),
                Token::EndOfLine,
                Token::ExclaimMark,
                Token::Kif,
                Token::Iden(String::from("test")),
                Token::BoLe,
                Token::IntLit(String::from("4")),
                Token::BoPlus,
                Token::IntLit(String::from("2")),
                Token::Comma,
                Token::Kset,
                Token::Iden(String::from("z")),
                Token::Kto,
                Token::IntLit(String::from("5")),
                Token::EndOfLine,
                Token::ExclaimMark,
                Token::Eof,
            ]
        );
        assert_eq!(ts.len(), res.1.len())
    }
}
