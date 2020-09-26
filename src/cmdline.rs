use std::env::args;
use std::fs;
use std::process::exit;
use std::process::Command;

use crate::analyze;
use crate::codegen;
use crate::lexer;
use crate::parser;

const ERROR: &str = "\x1B[31;1mERROR: \x1B[0m";

fn parse_cmds_to_code(input: String) -> String {
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
    code_text
}

#[derive(Debug)]
pub struct CmdArgInfo {
    debug: bool,
    input: String,
}

fn parse_cmd_line_opts() -> CmdArgInfo {
    let mut cmd_args_tmp = args().collect::<Vec<String>>();
    if cmd_args_tmp.len() == 1 {
        eprintln!("{}I need an input file.", ERROR);
        exit(1);
    }
    let mut cmd_line_args: Vec<&str> = cmd_args_tmp.iter_mut().map(|x| x.as_str()).collect();
    cmd_line_args.remove(0);
    let filename = cmd_line_args.remove(0);
    let input = match fs::read_to_string(&filename) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("{}Cannot read file: `{}`.", ERROR, filename);
            exit(1);
        }
    };
    let mut arg_info = CmdArgInfo {
        debug: false,
        input,
    };
    for i in cmd_line_args {
        match i {
            "-g" => arg_info.debug = true,
            e => arg_not_found(e),
        }
    }
    arg_info
}

fn arg_not_found(arg: &str) {
    eprintln!("{}Invalid option: {}", ERROR, arg);
    exit(1);
}

pub fn driver() {
    let opts = parse_cmd_line_opts();
    let code = parse_cmds_to_code(opts.input);
    // write the code to temp asm file
    if let Err(e) = fs::write("out.asm", code) {
        eprintln!("{}Cannot write assembly to temporary file: {}", ERROR, e);
        exit(1);
    }
    // assemble it
    if opts.debug {
        match Command::new("nasm")
            .arg("-felf64")
            .arg("-F")
            .arg("dwarf")
            .arg("-g")
            .arg("out.asm")
            .arg("-oout.o")
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("{}Nasm failed:", ERROR);
                    eprintln!(
                        "Nasm stderr:\n{}",
                        String::from_utf8(output.stderr).expect("invalid nasm stderr")
                    );
                    eprintln!(
                        "Nasm stdout:\n{}",
                        String::from_utf8(output.stdout).expect("invalid nasm stderr")
                    );
                    exit(1);
                }
            }
            Err(e) => {
                eprintln!("{}Failed to execute nasm: {}", ERROR, e);
                exit(1);
            }
        }
    } else {
        match Command::new("nasm")
            .arg("-felf64")
            .arg("out.asm")
            .arg("-oout.o")
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("{}nasm failed:", ERROR);
                    eprintln!(
                        "nasm stderr:\n{}",
                        String::from_utf8(output.stderr).expect("invalid nasm stderr")
                    );
                    eprintln!(
                        "nasm stdout:\n{}",
                        String::from_utf8(output.stdout).expect("invalid nasm stderr")
                    );
                    exit(1);
                }
            }
            Err(e) => {
                eprintln!("{}Failed to execute nasm: {}", ERROR, e);
                exit(1);
            }
        }
    }
    // link it
    match Command::new("ld")
        .arg("out.o")
        .arg("-o")
        .arg("a.out")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("{}ld failed:", ERROR);
                eprintln!(
                    "ld stderr:\n{}",
                    String::from_utf8(output.stderr).expect("invalid nasm stderr")
                );
                eprintln!(
                    "ld stdout:\n{}",
                    String::from_utf8(output.stdout).expect("invalid nasm stderr")
                );
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}Failed to execute ld: {}", ERROR, e);
            exit(1);
        }
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
