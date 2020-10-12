#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod analyse;
pub mod ast;
pub mod cmdline;
pub mod codegen;
pub mod errors;
pub mod lexer;
pub mod parser;
fn main() {
    cmdline::driver();
}
