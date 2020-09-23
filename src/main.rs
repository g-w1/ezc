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
    let input = "set z to 5. if z = 5,
            set a to 200.
            change a to 100 + a.
        !
    ";
    let output = tokenizer.lex(String::from(input));
    let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
    analyze::analize(&mut ast).unwrap();
    let mut code = codegen::Code::new();
    code.codegen(ast);
    println!("{}", code);
}
