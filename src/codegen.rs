use crate::ast::{Ast, AstNode, Expr, SetNode};
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

impl fmt::Display for Code {
    /// printing the asm to stdout. should be very easy to port to file because stdout is a file!!!
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// generate the code. dont deal with any of the sections
pub fn codegen(tree: Ast) -> Code {
    let mut code = Code {
        data: Data {
            instructions: Vec::new(),
        },
        text: Text {
            instructions: Vec::new(),
        },
        bss: Bss {
            instructions: Vec::new(),
        },
    };
    for node in tree.nodes {
        match node {
            AstNode::Set(stmt) => cgen_set_stmt(stmt, &mut code),
        }
    }
    code
}

/// code generation for a set statement
fn cgen_set_stmt(node: SetNode, code: &mut Code) {
    code.bss.instructions.push(format!("_{} resq 1", node.sete));
    match node.setor {
        Expr::Number(s) => {
            code.text
                .instructions
                .push(format!("mov qword [_{}], {}", node.sete, s));
        }
        // for recursive expressions
        // expr => code.text.instructions.push(cgen_expr(expr)),
    }
}
