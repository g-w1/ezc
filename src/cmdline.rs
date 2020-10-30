use std::env::args;
use std::fs;
use std::process::exit;
use std::process::Command;

use crate::analyse;
use crate::codegen;
use crate::lexer;
use crate::parser;

const ERROR: &str = "\x1B[31;1mERROR: \x1B[0m";

/// the driver function for the whole compiler
pub fn driver() {
    // generate the code
    let opts = parse_cmd_line_opts();
    let input = match fs::read_to_string(&opts.filename) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("{}Cannot read file: `{}`.", ERROR, opts.filename);
            exit(1);
        }
    };
    let code = parse_input_to_code(input, opts.library);
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
                .arg("-o")
                .arg(opts.filename.clone() + ".o"),
        );
    } else {
        command_run_error_printing(
            "nasm",
            Command::new("nasm")
                .arg("-felf64")
                .arg("out.asm")
                .arg("-o")
                .arg(opts.filename.clone() + ".o"),
        );
    }
    // link it
    if let Some(p) = opts.stdlib_path {
        if !opts.library && !opts.no_link {
            command_run_error_printing(
                "ld",
                Command::new("ld")
                    .arg(opts.filename.clone() + ".o")
                    .arg(p.as_str())
                    .arg("-o")
                    .arg("a.out"),
            );
        }
    } else {
        if !opts.library && !opts.no_link {
            command_run_error_printing(
                "ld",
                Command::new("ld")
                    .arg(opts.filename.clone() + ".o")
                    .arg("-o")
                    .arg("a.out"),
            );
        }
    }
    // remove temp files if not in debug mode
    if !opts.debug {
        if let Err(e) = fs::remove_file("out.asm") {
            eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
        }
    }
    if !opts.library && !opts.no_link {
        if let Err(e) = fs::remove_file(opts.filename.clone() + ".o") {
            eprintln!("{}Cannot remove temporary file: {}", ERROR, e);
        }
    }
}

fn parse_input_to_code(input: String, lib: bool) -> String {
    let mut tokenizer = lexer::Tokenizer::new();
    let output = tokenizer.lex(&input);
    if let Err(e) = output.0 {
        println!("{}{}", ERROR, e.print_the_error(&input));
        exit(1);
    }
    let code_text;
    let output = parser::parse(output.0.unwrap(), output.1);
    match output {
        Ok(mut res) => match analyse::analize(&mut res) {
            Ok(_) => {
                let mut code = codegen::Code::new();
                code.cgen(res);
                code_text = format!("{}", code.fmt(lib));
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
    no_link: bool,
    filename: String,
    library: bool,
    help: bool,
    stdlib_path: Option<String>,
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
            println!(
                "ezc

ezc version {}

Usage: ezc [file] [options] ...
Options:

-g                  Include Debug Info
-lib                Just compile the functions into a library/object (.o) file
-nolink             Just compile it into a .o file. Do not link. But this will contain _start.
-stdlib-path path   The path of the standard library object file so we can link to it.
-h | --help     Show This Help Message and Exit

To Report Bugs Go To: github.com/g-w1/ezc/issues/",
                env!("CARGO_PKG_VERSION")
            );
            exit(0);
        }
        _ => {}
    }
    // let input = match fs::read_to_string(&filename) {
    //     Ok(a) => a,
    //     Err(_) => {
    //         eprintln!("{}Cannot read file: `{}`.", ERROR, filename);
    //         exit(1);
    //     }
    // };
    let mut arg_info = CmdArgInfo {
        debug: false,
        help: true,
        library: false,
        no_link: false,
        stdlib_path: None,
        filename: filename.to_string(),
    };
    let mut args_iter = cmd_line_args.iter();
    while let Some(&i) = args_iter.next() {
        match i {
            "-g" => arg_info.debug = true,
            "-h" | "--help" => {
                arg_info.help = true;
                println!(
                    "ezc

ezc version {}

Usage: ezc [file] [options] ...
Options:

-g                  Include Debug Info
-lib                Just compile the functions into a library/object (.o) file
-nolink             Just compile it into a .o file. Do not link. But this will contain _start.
-stdlib-path path   The path of the standard library object file so we can link to it.
-h | --help     Show This Help Message and Exit

To Report Bugs Go To: github.com/g-w1/ezc/issues/",
                    env!("CARGO_PKG_VERSION")
                );
                exit(0);
            }
            "-lib" => arg_info.library = true,
            "-nolink" => arg_info.no_link = true,
            "-stdlib-path" => {
                arg_info.stdlib_path = Some({
                    if let Some(x) = args_iter.next() {
                        x.to_string()
                    } else {
                        arg_not_found("Need another option after -stdlib-path: The actual path");
                        exit(1)
                    }
                })
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
