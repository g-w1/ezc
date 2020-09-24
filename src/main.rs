#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod analyze;
pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;

fn main() {
    let mut tokenizer = lexer::Tokenizer::new();
    let input = "set x to 0.
    loop,
        if x < 5,
            change x to x + 1.
        !
        if x = 5,
            Break.
        !
    Break.
!";
    let output = tokenizer.lex(String::from(input));
    dbg!(&output.0);
    let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
    dbg!(&ast);
    analyze::analize(&mut ast).unwrap();
    let mut code = codegen::Code::new();
    code.codegen(ast);
    println!("{}", code);
}
