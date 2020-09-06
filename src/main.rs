#[warn(missing_docs)]
#[warn(missing_crate_level_docs)]
#[warn(missing_debug_implementations)]
pub mod lexer;

fn main() {
    let mut tokenizer = lexer::Tokenizer::new();
    let op = tokenizer.lex(String::from("Set x to 10. Set y to 5."));
    if let Ok(_) = op {
        println!("{:?}", tokenizer.output);
    }

}
