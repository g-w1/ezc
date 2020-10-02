use std::env::args;
use std::fs;
use std::process::exit;
use std::process::Command;

use crate::analyze;
use crate::codegen;
use crate::lexer;
use crate::parser;

const ERROR: &str = "\x1B[31;1mERROR: \x1B[0m";

const HELP_MESSAGE: &'static str = "ezc

Usage: ezc [file] [options] ...
Options:

-g              Include Debug Info
-h | --help     Show This Help Message and Exit

To Report Bugs Go To: github.com/g-w1/ezc/issues/";

/// the driver function for the whole compiler
pub fn driver() {
    // generate the code
    let opts = parse_cmd_line_opts();
    let code = parse_input_to_code(opts.input);
    // write the code to temp asm file
    fs::write("out.asm", code).unwrap_or_else(|e| {
        eprintln!("{}Cannot write assembly to temporary file: {}", ERROR, e);
        exit(1);
    });
    // assemble it
    if opts.debug {
        command_run_error_printing(
            "nasm",
            Command::new("nasm")
                .arg("-felf64")
                .arg("-F")
                .arg("dwarf")
                .arg("-g")
                .arg("out.asm")
                .arg("-oout.o"),
        );
    } else {
        command_run_error_printing(
            "nasm",
            Command::new("nasm")
                .arg("-felf64")
                .arg("out.asm")
                .arg("-oout.o"),
        );
    }
    // link it
    command_run_error_printing("ld", Command::new("ld").arg("out.o").arg("-o").arg("a.out"));
    // remove temp files if not in debug mode
    if !opts.debug {
        if let Err(e) = fs::remove_file("out.asm") {
            eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
        }
    }
    if let Err(e) = fs::remove_file("out.o") {
        eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
    }
}

fn parse_input_to_code(input: String) -> String {
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer.lex(&input);
    if let Err(e) = output.0 {
        println!("{}{}", ERROR, e.print_the_error(&input));
        exit(1);
    }
    let code_text;
    let output = parser::parse(output.0.unwrap(), output.1);
    match output {
        Ok(mut res) => match analyze::analize(&mut res) {
            Ok(_) => {
                dbg!(&res);
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
            println!("{}{}", ERROR, e.print_the_error(&input));
            exit(1);
        }
    }
    code_text
}

struct CmdArgInfo {
    debug: bool,
    input: String,
    help: bool,
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
    match filename {
        "-h" | "--help" => {
            println!("{}", HELP_MESSAGE);
            exit(0);
        }
        _ => {}
    }
    let input = match fs::read_to_string(&filename) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("{}Cannot read file: `{}`.", ERROR, filename);
            exit(1);
        }
    };
    let mut arg_info = CmdArgInfo {
        debug: false,
        help: true,
        input,
    };
    for i in cmd_line_args {
        match i {
            "-g" => arg_info.debug = true,
            "-h" | "--help" => {
                arg_info.help = true;
                println!("{}", HELP_MESSAGE);
                exit(0);
            }
            e => arg_not_found(e),
        }
    }
    arg_info
}

fn arg_not_found(arg: &str) {
    eprintln!("{}Invalid option: {}", ERROR, arg);
    exit(1);
}

fn command_run_error_printing(cmd_name: &'static str, cmd: &mut Command) {
    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("{}{} failed:", ERROR, cmd_name);
                eprintln!(
                    "{} stderr:\n{}",
                    cmd_name,
                    String::from_utf8(output.stderr).expect("invalid nasm stderr")
                );
                eprintln!(
                    "{} stdout:\n{}",
                    cmd_name,
                    String::from_utf8(output.stdout).expect("invalid nasm stderr")
                );
                exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}Failed to execute {}: {}", ERROR, cmd_name, e);
            exit(1);
        }
    }
}
