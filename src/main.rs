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
    let input = "set z to 5. if 5 > 4,
            set a to 6.
            change a to 4.
        !
    set a to 6. ";
    // println!("input: {}", &input);
    let output = tokenizer.lex(String::from(input));
    // println!("lexed: {:?}", output);
    let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
    let ast = parser.parse(true).unwrap();
    analyze::analize(&ast).unwrap();
    println!("ast: {:#?}", ast);
    // let mut code = codegen::Code::new();
    // code.codegen(ast);
    // println!("{}", code);
}
