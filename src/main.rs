#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod ast;
pub mod lexer;
pub mod parser;

fn main() {
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer
        .lex(String::from("Set x to 10. set y to 5."))
        .unwrap();
    // println!("{:?}", &output);
    let mut parser = parser::Parser::new(output);
    let ast = parser.parse().unwrap();
    println!("{:#?}", ast);
}
