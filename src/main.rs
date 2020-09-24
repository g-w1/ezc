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
    // let input = "set y to 4. set z to 6. if z >= y,
    //     set a to y.
    //     change y to z.
    //     if a = 4,
    //         change z to a.
    //     !
    // !";
    let input = "Set y to 5. Set x to (y+5 - 10)+y-15. set z to x + 4. set res_of_bop to x - z < 10.";
    let output = tokenizer.lex(String::from(input));
    let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
    analyze::analize(&mut ast).unwrap();
    let mut code = codegen::Code::new();
    code.codegen(ast);
    println!("{}", code);
}
