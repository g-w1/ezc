#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod analyze;
pub mod ast;
pub mod codegen;
pub mod errors;
pub mod lexer;
pub mod parser;
use std::env::args;
use std::fs;

const ERROR: &str = "\x1B[31;1mERROR: \x1B[0m";

fn main() {
    let input;
    if let Some(filename) = args().nth(1) {
        input = match fs::read_to_string(&filename) {
            Ok(a) => a,
            Err(_) => {
                eprintln!("{} Cannot read file: `{}`.", ERROR, filename);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!(
            "{}I need the first command line arg to be the file to compile.",
            ERROR
        );
        std::process::exit(1);
    }
    // let input = "set z to 4. change z to 3. set x to z + z.";
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer.lex(input);
    if let Err(e) = output.0 {
        println!("{}{}", ERROR, e);
        std::process::exit(1);
    }
    match parser::parse(output.0.unwrap(), output.1) {
        Ok(mut res) => match analyze::analize(&mut res) {
            Ok(_) => {
                let mut code = codegen::Code::new();
                code.codegen(res);
                println!("{}", code);
            }
            Err(e) => {
                println!("{}{}", ERROR, e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("{}{}", ERROR, e);
            std::process::exit(1);
        }
    }
}
