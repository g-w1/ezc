pub enum Token {
    Kword,
    Iden(String),
    Number(String),
}

pub type TokenStream = Vec<Token>;
pub fn lex(input: &str) -> TokenStream {

}
