#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod analyze;
pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
use std::env::args;
use std::fs;

fn main() {
    let input;
    if let Some(filename) = args().nth(1) {
        input = match fs::read_to_string(&filename) {
            Ok(a) => a,
            Err(_) => {
                eprintln!("Error: Cannot read file: `{}`.", filename);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: I need the first command line arg to be the file to compile.");
        std::process::exit(1);
    }
    // let input = "set z to 4. change z to 3. set x to z + z.";
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer.lex(input);
    let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
    analyze::analize(&mut ast).unwrap();
    let mut code = codegen::Code::new();
    code.codegen(ast);
    println!("{}", code);
}
