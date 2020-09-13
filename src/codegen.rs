//! code generation for the compiler

use crate::ast::{Ast, AstNode, BinOp, Expr};
use std::fmt;
/// section .bss
#[derive(Debug)]
pub struct Bss {
    pub instructions: Vec<String>,
}

/// section .data
#[derive(Debug)]
pub struct Data {
    pub instructions: Vec<String>,
}

/// section .text
#[derive(Debug)]
pub struct Text {
    pub instructions: Vec<String>,
}

/// represent asm
#[derive(Debug)]
pub struct Code {
    data: Data,
    bss: Bss,
    text: Text,
}

/// a helper function to provide `qword [_varname]` from `varname`
fn qword_deref_helper(input: String) -> String {
    format!("qword [_{}]", input)
}

impl Code {
    pub fn new() -> Self {
        Code {
            data: Data {
                instructions: Vec::new(),
            },
            text: Text {
                instructions: Vec::new(),
            },
            bss: Bss {
                instructions: Vec::new(),
            },
        }
    }
    /// generate the code. dont deal with any of the sections
    pub fn codegen(self: &mut Self, tree: Ast) {
        for node in tree.nodes {
            match node {
                AstNode::SetOrChange {
                    sete,
                    setor,
                    change,
                } => self.cgen_set_or_change_stmt(sete, setor, change),
            }
        }
    }

    /// code generation for a set or change stmt. it is interpreted as change if change is true
    fn cgen_set_or_change_stmt(self: &mut Self, sete: String, setor: Expr, change: bool) {
        if !change {
            self.bss.instructions.push(format!("_{} resq 1", sete));
        }
        match setor {
            Expr::Number(s) => {
                // if it is just a number push it to .text here
                self.text
                    .instructions
                    .push(format!("mov {}, {}", qword_deref_helper(sete), s));
            }
            Expr::Iden(s) => {
                // if it is another iden then move the val to it
                self.text.instructions.push(format!(
                    "mov {}, {}",
                    qword_deref_helper(sete),
                    qword_deref_helper(s)
                ));
            }
            // for recursive expressions
            Expr::BinOp { lhs, rhs, op } => {
                let reg = "rax"; // TODO may need to change if "rax" is used
                // self.cgen_expr(*lhs, op, *rhs, reg);
            }
        }
    }

    // /// A function to recursively generate code for expressions. TODO make work
    // fn cgen_expr(self: &mut Self, lhs: Expr, op: BinOp, rhs: Expr, reg: &str) {
    //     match lhs {
    //         Expr::Number(n) {

    //         }
    //     }
    //     match op {
    //         BinOp::Add => {
    //             self.text.instructions.push()
    //         }
    //         BinOp::Sub => {

    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn codegen_set_stmt() {
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . set   test to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut code = codegen::Code::new();
        code.codegen(ast);
        let correct_code = "global _start
section .text
_start:
mov qword [_x], 10
mov qword [_y], 5
mov qword [_test], 445235
mov rax, 60
xor rdi, rdi
syscall
section .bss
_x resq 1
_y resq 1
_test resq 1
";
        assert_eq!(format!("{}", code), correct_code);
    }
    #[test]
    fn codegen_change_stmt() {
        use crate::analyze;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . change   x to 445235 .";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse().unwrap();
        let mut analizer = analyze::Analyser::new();
        analizer.analyze(&ast).unwrap();
        let mut code = codegen::Code::new();
        code.codegen(ast);
        let correct_code = "global _start
section .text
_start:
mov qword [_x], 10
mov qword [_y], 5
mov qword [_x], 445235
mov rax, 60
xor rdi, rdi
syscall
section .bss
_x resq 1
_y resq 1
";
        assert_eq!(format!("{}", code), correct_code);
    }
}

impl fmt::Display for Code {
    /// printing the asm to stdout. should be very easy to port to file because stdout is a file!!!
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // adding the sections
        if self.text.instructions.clone().len() > 0 {
            writeln!(f, "global _start")?;
            writeln!(f, "section .text")?;
            writeln!(f, "_start:")?;
            for i in self.text.instructions.clone() {
                writeln!(f, "{}", i)?;
            }
            // exit 0
            writeln!(
                f,
                "mov rax, 60
xor rdi, rdi
syscall"
            )?;
        }
        if self.data.instructions.clone().len() > 0 {
            writeln!(f, "section .data")?;
            for i in self.data.instructions.clone() {
                writeln!(f, "{}", i)?;
            }
        }
        if self.bss.instructions.clone().len() > 0 {
            writeln!(f, "section .bss")?;
            for i in self.bss.instructions.clone() {
                writeln!(f, "{}", i)?;
            }
        }

        Ok(())
    }
}
