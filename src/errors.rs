use crate::analyse::AnalysisError;
use crate::lexer::{LexError, Token, Token::*};
use crate::parser::ParserError;
use std::fmt;

impl LexError {
    pub fn print_the_error(&self, input_code: &str) -> String {
        match self {
            LexError::UnexpectedChar(char_unex, pos) => format!(
                "Lexer Error: Unexpected Char: `{}`\n{}",
                char_unex,
                special_error_printing_with_caret(&input_code, pos)
            ),
        }
    }
}

impl ParserError {
    /// a method to print a parser error
    pub fn print_the_error(&self, input_code: &str) -> String {
        match self {
            ParserError::ExectedOneFoundAnother {
                expected,
                found,
                pos,
            } => format!(
                "Parser Error: expected {}, found {}\n{}",
                expected,
                found,
                special_error_printing_with_caret(&input_code, &pos)
            ),
        }
    }
}

/// special error fancy printing
fn special_error_printing_with_caret(input_code: &str, pos: &u32) -> String {
    let mut until_pos_counter = 0;
    let mut special_line: &str = "";
    let mut special_row = 0;
    let mut special_col = 0;
    for (i, line) in input_code.lines().enumerate() {
        until_pos_counter += line.len() + 1;
        if until_pos_counter >= *pos as usize {
            special_line = line;
            special_row = i;
            special_col = *pos as usize + line.len() - until_pos_counter + 2;
            break;
        }
    }
    format!(
        "{}:{}:\n{}\n{}",
        special_row + 1,
        special_col,
        special_line,
        up_caret(&special_col)
    )
}

/// A function to put an up caret under a bad code sample for coolness
fn up_caret(num: &usize) -> String {
    let mut res = String::new();
    for _ in 0..(num - 1) {
        res += " ";
    }
    res += "\x1B[31;1m^\x1B[0m";
    res
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnalysisError::FuncAlreadyExists(s) => write!(f, "Analysis Error: function is declared twice: {}", s),
            AnalysisError::SameArgForFunction(s) => match s {
                    crate::ast::Type::Num(name) => write!(f, "Analysis Error: the same arg was used in a function definition: {}", name) ,
                    crate::ast::Type::ArrNum(name, num) => write!(f, "Analysis Error: the same arg was used in a function definition: [{}]{}", num,name),
                }
            AnalysisError::BreakWithoutLoop => write!(f, "Analysis Error: there was a break statement outside of a loop."),
            // TODO add info about which var it was
            AnalysisError::DoubleSet(v) => write!(f, "Analysis Error: the same variable `{}` was set twice. \nHint: use `change` to change the value of the variable once it is set: Ex `set x to 0. change x to 4.`", v),
            AnalysisError::VarNotExist(v) => write!(f, "Analysis Error: the variable `{}` was used, but it doesn't exist in this scope.", v),
            // TODO get info on whch num
            AnalysisError::NumberTooBig(num) => write!(f, "Analysis Error: Number too big: `{}`", num),
            AnalysisError::SetInLoop => write!(f, "A set statement was used in a loop. Not allowed."),
            AnalysisError::ReturnOutSideOfFunc => write!(f, "A return statement was used outside of a function. Not allowed."),
            AnalysisError::FuncCalledWithWrongArgsType(name, should, had) => write!(f, "The function {} was called with {:?} args but it takes {:?} args.", name, had,should), // TODO impliment display but thats later
            AnalysisError::FuncCalledButNoExist(name) => write!(f, "The function {} was called but it does not exist.",name),

        }
    }
}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EndOfLine => write!(f, "'.'"),
            Kset => write!(f, "\"set\""),
            Kextern => write!(f, "\"external\""),
            Kexport => write!(f, "\"export\""),
            Kchange => write!(f, "\"change\""),
            Comma => write!(f, "','"),
            Kif => write!(f, "\"if\""),
            Kto => write!(f, "\"to\""),
            Kloop => write!(f, "\"loop\""),
            Kfunc => write!(f, "\"func\""),
            Kreturn => write!(f, "\"return\""),
            Kbreak => write!(f, "\"break\""),
            Rparen => write!(f, "')'"),
            Lparen => write!(f, "'('"),
            ExclaimMark => write!(f, "'!'"),
            Iden(i) => write!(f, "\"{}\"", i),
            IntLit(i) => write!(f, "{}", i),
            Eof => write!(f, "EOF"),
            BoPlus => write!(f, "'+'"),
            BoMinus => write!(f, "'-'"),
            BoMul => write!(f, "'*'"),
            BoL => write!(f, "'<'"),
            BoG => write!(f, "'>'"),
            BoLe => write!(f, "'<='"),
            BoGe => write!(f, "'>='"),
            BoE => write!(f, "'='"),
            BoNe => write!(f, "'!='"),
            BoAnd => write!(f, "'and'"),
            BoOr => write!(f, "'or'"),
            OpenBrak => write!(f, "'['"),
            CloseBrak => write!(f, "']'"),
        }
    }
}
