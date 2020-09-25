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
use std::process::exit;
use std::process::Command;

const ERROR: &str = "\x1B[31;1mERROR: \x1B[0m";

fn main() {
    let (code, opts) = parse_cmds_to_code();
    // write the code to temp asm file
    if let Err(e) = fs::write("out.asm", code) {
        eprintln!("{}Cannot write assembly to temporary file: {}", ERROR, e);
        exit(1);
    }
    // assemble it
    if let Err(e) = Command::new("nasm")
        .arg("-felf64")
        .arg("-F")
        .arg("dwarf")
        .arg("-g")
        .arg("out.asm")
        .arg("-oout.o")
        .output()
    {
        eprintln!("{}Failed to execute nasm: {}", ERROR, e);
        exit(1);
    }
    // link it
    if let Err(e) = Command::new("ld")
        .arg("out.o")
        .arg("-o")
        .arg("a.out")
        .output()
    {
        eprintln!("{}Failed to execute ld: {}", ERROR, e);
        exit(1);
    }
    // remove temp files
    if !opts.debug {
        if let Err(e) = fs::remove_file("out.asm") {
            eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
        }
    }
    if let Err(e) = fs::remove_file("out.o") {
        eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
    }
}
fn parse_cmds_to_code() -> (String, CmdArgInfo) {
    let input;
    if let Some(filename) = args().nth(1) {
        input = match fs::read_to_string(&filename) {
            Ok(a) => a,
            Err(_) => {
                eprintln!("{}Cannot read file: `{}`.", ERROR, filename);
                exit(1);
            }
        }
    } else {
        eprintln!(
            "{}I need the first command line arg to be the file to compile.",
            ERROR
        );
        exit(1);
    }
    // let input = "set z to 4. change z to 3. set x to z + z.";
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer.lex(input);
    if let Err(e) = output.0 {
        println!("{}{}", ERROR, e);
        exit(1);
    }
    let code_text;
    match parser::parse(output.0.unwrap(), output.1) {
        Ok(mut res) => match analyze::analize(&mut res) {
            Ok(_) => {
                let mut code = codegen::Code::new();
                code.codegen(res);
                code_text = format!("{}", code);
            }
            Err(e) => {
                println!("{}{}", ERROR, e);
                exit(1);
            }
        },
        Err(e) => {
            println!("{}{}", ERROR, e);
            exit(1);
        }
    }
    let mut arg_info = CmdArgInfo { debug: false };
    if let Some(s) = args().nth(2) {
        if let "-g" = s.as_str() {
            arg_info.debug = true;
        }
    }
    (code_text, arg_info)
}

struct CmdArgInfo {
    debug: bool,
}
