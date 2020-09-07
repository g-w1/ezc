#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod ast;
pub mod lexer;
pub mod parser;

fn main() {
    let mut tokenizer = lexer::Tokenizer::new();
    let op = tokenizer.lex(String::from("Set x to 10. Set y to 5."));
    if let Ok(output) = op {
        println!("{:?}", output);
    }
}
