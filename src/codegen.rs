//! code generation for the compiler

use crate::ast::{Ast, AstNode, BinOp, Expr};
use std::collections::HashMap;
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
    number_for_mangling: u32,
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
            number_for_mangling: 0,
        }
    }
    /// generate the code. dont deal with any of the sections
    pub fn codegen(self: &mut Self, tree: Ast) {
        for node in tree {
            match node {
                AstNode::SetOrChange {
                    sete,
                    setor,
                    change,
                } => self.cgen_static_set_or_change_stmt(sete, setor, change),
                AstNode::If { guard, body } => self.cgen_if_stmt(guard, body),
            }
        }
    }
    /// code gen for if stmt. uses stack based allocation
    fn cgen_if_stmt(self: &mut Self, guard: Expr, body: Ast) {
        let vars: HashMap<String, u32> = get_all_var_decls(&body);
        let mem_len = vars.len() * 8;
        self.text.instructions.push(String::from("push rbp"));
        self.text.instructions.push(String::from("mov rbp, rsp"));
        self.text.instructions.push(format!("sub rsp, {}", mem_len)); // allocate locals
    }
    /// code generation for a set or change stmt. it is interpreted as change if change is true
    fn cgen_static_set_or_change_stmt(self: &mut Self, sete: String, setor: Expr, change: bool) {
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
                let reg = "r8";
                self.cgen_expr(*lhs, op, *rhs);
                self.text.instructions.push(format!("pop {}", reg));
                self.text
                    .instructions
                    .push(format!("mov {}, {}", qword_deref_helper(sete), reg));
            }
        }
    }

    /// A function to recursively generate code for expressions.
    fn cgen_expr(self: &mut Self, lhs: Expr, op: BinOp, rhs: Expr) {
        // let lhs_is_leaf: bool = matches!(lhs, Expr::Iden(_) | Expr::Number(_));
        // let rhs_is_leaf: bool = matches!(rhs, Expr::Iden(_) | Expr::Number(_));
        let cloned_rhs = rhs.clone();
        let cloned_lhs = lhs.clone();
        match (lhs, rhs) {
            // base case
            //       op
            //     /    \
            //  num     num
            (Expr::Iden(_), Expr::Number(_))
            | (Expr::Number(_), Expr::Iden(_))
            | (Expr::Number(_), Expr::Number(_))
            | (Expr::Iden(_), Expr::Iden(_)) => {
                self.text
                    .instructions
                    .push(format!("push {}", cloned_lhs.get_display_asm()));
                self.text
                    .instructions
                    .push(format!("push {}", cloned_rhs.get_display_asm()));
                self.text
                    .instructions
                    .extend_from_slice(&op.cgen_for_stack(&mut self.number_for_mangling));
                // // update it by 2 because 2 was used and dont wanna pass mut
                // self.number_for_mangling += 2;
            }
            //        op
            //      /    \
            //     op     num
            //    /  \
            // num    num
            (
                Expr::BinOp {
                    lhs: reclhs,
                    op: recop,
                    rhs: recrhs,
                },
                Expr::Iden(_),
            )
            | (
                Expr::BinOp {
                    lhs: reclhs,
                    op: recop,
                    rhs: recrhs,
                },
                Expr::Number(_),
            ) => {
                self.cgen_expr(*reclhs, recop, *recrhs);
                self.text
                    .instructions
                    .push(format!("push {}", cloned_rhs.get_display_asm()));
                self.text
                    .instructions
                    .extend_from_slice(&op.cgen_for_stack(&mut self.number_for_mangling));
                self.number_for_mangling += 2;
            }
            //        op
            //      /    \
            //     num     op
            //            /  \
            //         num    num
            (
                Expr::Iden(_),
                Expr::BinOp {
                    lhs: reclhs,
                    op: recop,
                    rhs: recrhs,
                },
            )
            | (
                Expr::Number(_),
                Expr::BinOp {
                    lhs: reclhs,
                    op: recop,
                    rhs: recrhs,
                },
            ) => {
                self.text
                    .instructions
                    .push(format!("push {}", cloned_rhs.get_display_asm()));
                self.cgen_expr(*reclhs, recop, *recrhs);
                self.text
                    .instructions
                    .extend_from_slice(&op.cgen_for_stack(&mut self.number_for_mangling));
                self.number_for_mangling += 2;
            }
            (
                Expr::BinOp {
                    lhs: lreclhs,
                    op: lrecop,
                    rhs: lrecrhs,
                },
                Expr::BinOp {
                    lhs: rreclhs,
                    op: rrecop,
                    rhs: rrecrhs,
                },
            ) => {
                self.cgen_expr(*lreclhs, lrecop, *lrecrhs);
                self.cgen_expr(*rreclhs, rrecop, *rrecrhs);
                self.text
                    .instructions
                    .extend_from_slice(&op.cgen_for_stack(&mut self.number_for_mangling));
                self.number_for_mangling += 2;
            }
        }
    }
}

impl BinOp {
    /// takes 2 things on the stack. pops them, does an arg and then pushes the result
    fn cgen_for_stack(self: &Self, num_for_mangling: &mut u32) -> [String; 4] {
        match self {
            Self::Add => [
                String::from("pop r8"),
                String::from("pop r9"),
                String::from("add r8, r9"),
                String::from("push r8"),
            ],
            Self::Sub => [
                String::from("pop r8"),
                String::from("pop r9"),
                String::from("sub r9, r8"),
                String::from("push r9"),
            ],
            Self::Gt => {
                *num_for_mangling += 2;
                [
                    String::from("pop r9"),
                    String::from("pop r8"),
                    format!(
                        "cmp r9, r8\njg .IF_{}\njle .ELSE_{}",
                        num_for_mangling, num_for_mangling
                    ),
                    format!(
                        ".IF_{}:\npush 1\n.ELSE_{}\npush 0",
                        num_for_mangling, num_for_mangling
                    ),
                ]
            }
            Self::Lt => {
                *num_for_mangling += 2;
                [
                    String::from("pop r9"),
                    String::from("pop r8"),
                    format!(
                        "cmp r9, r8\njl .IF_{}\njge .ELSE_{}",
                        num_for_mangling, num_for_mangling
                    ),
                    format!(
                        ".IF_{}:\npush 1\n.ELSE_{}\npush 0",
                        num_for_mangling, num_for_mangling
                    ),
                ]
            }
            Self::Equ => {
                *num_for_mangling += 2;
                [
                    String::from("pop r9"),
                    String::from("pop r8"),
                    format!(
                        "cmp r9, r8\nje .IF_{}\njne .ELSE_{}",
                        num_for_mangling, num_for_mangling
                    ),
                    format!(
                        ".IF_{}:\npush 1\n.ELSE_{}\npush 0",
                        num_for_mangling, num_for_mangling
                    ),
                ]
            }
            Self::Lte => {
                *num_for_mangling += 2;
                [
                    String::from("pop r9"),
                    String::from("pop r8"),
                    format!(
                        "cmp r9, r8\njle .IF_{}\njg .ELSE_{}",
                        num_for_mangling, num_for_mangling
                    ),
                    format!(
                        ".IF_{}:\npush 1\n.ELSE_{}\npush 0",
                        num_for_mangling, num_for_mangling
                    ),
                ]
            }
            Self::Gte => {
                *num_for_mangling += 2;
                [
                    String::from("pop r9"),
                    String::from("pop r8"),
                    format!(
                        "cmp r9, r8\njge .IF_{}\njl .ELSE_{}",
                        num_for_mangling, num_for_mangling
                    ),
                    format!(
                        ".IF_{}:\npush 1\n.ELSE_{}\npush 0",
                        num_for_mangling, num_for_mangling
                    ),
                ]
            }
        }
    }
}

impl Expr {
    /// if its a num or iden give how to display it deferenecd
    fn get_display_asm(self: &Self) -> String {
        match self {
            Self::Iden(a) => qword_deref_helper(a.to_owned()),
            Self::Number(n) => n.to_owned(),
            _ => unreachable!(),
        }
    }
}

/// get all the variable declarations in a block.
fn get_all_var_decls(tree: &Ast) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    for (i, node) in tree.iter().enumerate() {
        match node {
            AstNode::SetOrChange {
                sete,
                setor: _,
                change: false,
            } => {
                map.insert(sete.clone(), i as u32);
            }
            _ => {}
        }
    }
    map.clone()
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
        let ast = parser.parse(true).unwrap();
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
        let ast = parser.parse(true).unwrap();
        analyze::analize(&ast).unwrap();
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
    #[test]
    fn codegen_expr() {
        use crate::analyze;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set y to 5. Set x to (y+5 - 10)+y-15. set z to x + 4.";
        let output = tokenizer.lex(String::from(input));
        let mut parser = parser::Parser::new(output.0.unwrap(), output.1);
        let ast = parser.parse(true).unwrap();
        // TODO spelling. one is spelled with y and other with i
        analyze::analize(&ast).unwrap();
        let mut code = codegen::Code::new();
        code.codegen(ast);
        let correct_code = "global _start
section .text
_start:
mov qword [_y], 5
push qword [_y]
push 5
pop r8
pop r9
add r8, r9
push r8
push 10
pop r8
pop r9
sub r9, r8
push r9
push qword [_y]
pop r8
pop r9
add r8, r9
push r8
push 15
pop r8
pop r9
sub r9, r8
push r9
pop r8
mov qword [_x], r8
push qword [_x]
push 4
pop r8
pop r9
add r8, r9
push r8
pop r8
mov qword [_z], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
_y resq 1
_x resq 1
_z resq 1
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
