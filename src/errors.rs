use crate::analyze::AnalysisError;
use crate::lexer::LexError;
use crate::parser::ParserError;
use std::fmt;

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(c) => write!(f, "Lexer Error: Unexpected Char: `{}`.", c),
        }
    }
}
impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // TODO get cords to actually work
            ParserError::ExectedOneFoundAnother {
                expected: ex,
                found: fd,
                coords: _,
            } => write!(f, "Parser Error: expected {:?}, found {:?}.", ex, fd),
        }
    }
}
impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnalysisError::BreakWithoutLoop => write!(f, "Analysis Error: there was a break statement outside of a loop."),
            // TODO add info about which var it was
            AnalysisError::DoubleSet => write!(f, "Analysis Error: the same variable was set twice. \nHint: use `change` to change the value of the variable: Ex `set x to 0. change x to 4.`"),
            AnalysisError::VarNotExist(v) => write!(f, "Analysis Error: the variable {} was used, but it doesn't exist.", v),
            // TODO get info on whch num
            AnalysisError::NumberTooBig => write!(f, "Analysis Error: Number too big."),
            AnalysisError::SetInLoop => write!(f, "A set statement was used in a loop. Not allowed.")
        }
    }
}
