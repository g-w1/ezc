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
    let input = "Set y to 5. Set   x to (y+5 - 10)+y-15. set z to x + 4.";
    // println!("input: {}", &input);
    let output = tokenizer.lex(String::from(input));
    // println!("lexed: {:?}", output);
    let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
    let ast = parser.parse().unwrap();
    // println!("ast: {:#?}", ast);
    let mut analizer = analyze::Analyser::new();
    analizer.analyze(&ast).unwrap();
    let mut code = codegen::Code::new();
    code.codegen(ast);
    println!("{}", code);
}
