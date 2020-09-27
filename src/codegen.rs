//! code generation for the compiler

use crate::ast::{AstNode, AstRoot, BinOp, Expr};
use std::collections::HashMap;
use std::collections::HashSet;
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
    initalized_local_vars: HashMap<String, u32>,
    number_for_mangling: u32,
    stack_p_offset: u32,
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
            stack_p_offset: 0,
            initalized_local_vars: HashMap::new(),
        }
    }
    /// generate the code. dont deal with any of the sections
    pub fn codegen(self: &mut Self, tree: AstRoot) {
        for var in tree.static_vars.unwrap() {
            self.bss.instructions.push(format!("_{} resq 1", var));
        }
        for node in tree.tree {
            match node {
                AstNode::SetOrChange {
                    sete,
                    setor,
                    change: _,
                } => self.cgen_set_or_change_stmt(sete, setor),
                AstNode::If {
                    guard,
                    body,
                    vars_declared,
                } => self.cgen_if_stmt(guard, vars_declared.unwrap(), body, None), // we unwrap because it was analised
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                _ => unreachable!(),
            }
        }
    }
    /// code gen for if stmt. uses stack based allocation
    fn cgen_if_stmt(
        self: &mut Self,
        guard: Expr,
        vars: HashMap<String, u32>,
        body: Vec<AstNode>,
        loop_num: Option<u32>,
    ) {
        ///////////////////////////////////// EVALUATE THE ACTUAL BOOL //////////////////////////////
        let our_number_for_mangling = self.number_for_mangling;
        match guard {
            Expr::BinOp { lhs, op, rhs } => {
                let reg = "r8";
                self.cgen_expr(*lhs, op, *rhs);
                self.text.instructions.push(format!("pop {}", reg));
                self.stack_p_offset -= 1;
                self.text.instructions.push(format!("cmp {}, 1", reg));
            }
            Expr::Number(n) => {
                self.text.instructions.push(format!("cmp {}, 1", n)); // TODO really easy optimisation by just parsing num at compile time. but right now this is easier. premature optimisation is the start of all evil
            }
            Expr::Iden(_) => {
                self.text
                    .instructions
                    .push(format!("cmp {}, 1", self.get_display_asm(&guard)));
            }
        }
        self.text
            .instructions
            .push(format!("je .IF_BODY_{}", our_number_for_mangling));
        self.text
            .instructions
            .push(format!("jne .IF_END_{}", our_number_for_mangling));
        ///////////////////////// THE BODY OF THE IF STMT ////////////////////////////////////////////////////////
        self.text
            .instructions
            .push(format!(".IF_BODY_{}", our_number_for_mangling));
        ///////////// ALLOCATION FOR THE IF STMT //////////////////////////////
        let mut double_keys: HashSet<String> = HashSet::new();
        // the use of this is something like this. when something is declared inside a block and also needs to be out of the block. cant drop it. but do drop the memory. just not drop it from the initial hashmap:
        // set z to 5. if z = 5,
        //     set a to 5.
        //     if a > 4,
        //         set test to a.
        //     !
        //     set test to 5.
        // !

        let mem_len = vars.len();
        self.stack_p_offset += mem_len as u32;
        self.text
            .instructions
            .push(format!("sub rsp, {} * 8", mem_len)); // allocate locals
        for (varname, place) in vars.to_owned() {
            if self.initalized_local_vars.contains_key(&varname) {
                double_keys.insert(varname.clone());
            }
            self.initalized_local_vars
                .insert(varname, self.stack_p_offset as u32 - place);
        }
        for node in body {
            match node {
                AstNode::If {
                    body,
                    guard,
                    vars_declared,
                } => self.cgen_if_stmt(guard, vars_declared.unwrap(), body, None),
                AstNode::SetOrChange {
                    sete,
                    setor,
                    change: _,
                } => self.cgen_set_or_change_stmt(sete, setor),
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                AstNode::Break => self
                    .text
                    .instructions
                    .push(format!("jmp .END_LOOP_{}", loop_num.unwrap())),
            }
        }
        ///////////////////////// DEALLOCATION FOR THE VARS DECLARED INSIDE THE STMT ////////////////////////////
        for (var, _) in vars {
            if !double_keys.contains(&var) {
                self.initalized_local_vars.remove(&var);
            }
        }
        self.text
            .instructions
            .push(format!("add rsp, {} * 8", mem_len)); // deallocate locals?
        self.text
            .instructions
            .push(format!(".IF_END_{}", our_number_for_mangling));
        self.number_for_mangling += 1;
    }
    /// code generation for a loop. very easy
    fn cgen_loop_stmt(self: &mut Self, body: Vec<AstNode>) {
        let our_number_for_mangling = self.number_for_mangling;
        self.number_for_mangling += 1;
        self.text
            .instructions
            .push(format!(".START_LOOP_{}", our_number_for_mangling));
        for node in body {
            match node {
                AstNode::SetOrChange {
                    sete,
                    change: true,
                    setor,
                } => self.cgen_set_or_change_stmt(sete, setor),
                AstNode::If {
                    guard,
                    body,
                    vars_declared,
                } => self.cgen_if_stmt(
                    guard,
                    vars_declared.unwrap(),
                    body,
                    Some(our_number_for_mangling),
                ),
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                AstNode::Break => self
                    .text
                    .instructions
                    .push(format!("jmp .END_LOOP_{}", our_number_for_mangling)),
                AstNode::SetOrChange {
                    sete: _,
                    setor: _,
                    change: false,
                } => unreachable!(),
            }
        }
        self.text.instructions.push(format!(
            "jmp .START_LOOP_{}\n.END_LOOP_{}",
            our_number_for_mangling, our_number_for_mangling
        ))
    }
    /// code generation for a set or change stmt. it is interpreted as change if change is true
    fn cgen_set_or_change_stmt(self: &mut Self, sete: String, setor: Expr) {
        match setor {
            Expr::Number(s) => {
                // if it is just a number push it to .text here
                self.text.instructions.push(format!(
                    "mov {}, {}",
                    self.get_display_asm(&Expr::Iden(sete)),
                    s
                ));
            }
            Expr::Iden(s) => {
                // if it is 2 things that are in stack memory, then do it in 2 places bc cant copy mem directoly
                self.text.instructions.push(format!(
                    "mov r8, {}\nmov {}, r8",
                    self.get_display_asm(&Expr::Iden(s)),
                    self.get_display_asm(&Expr::Iden(sete))
                ));
            }
            // for recursive expressions
            Expr::BinOp { lhs, rhs, op } => {
                let reg = "r8";
                self.cgen_expr(*lhs, op, *rhs);
                self.text.instructions.push(format!("pop {}", reg));
                self.stack_p_offset -= 1;
                self.text.instructions.push(format!(
                    "mov {}, {}",
                    self.get_display_asm(&Expr::Iden(sete)),
                    reg
                ));
            }
        }
    }

    /// A function to recursively generate code for expressions.
    fn cgen_expr(self: &mut Self, lhs: Expr, op: BinOp, rhs: Expr) {
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
                    .push(format!("push {}", self.get_display_asm(&cloned_lhs)));
                self.stack_p_offset += 1;
                self.text
                    .instructions
                    .push(format!("push {}", self.get_display_asm(&cloned_rhs)));
                self.stack_p_offset += 1;
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
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
                    .push(format!("push {}", self.get_display_asm(&cloned_rhs)));
                self.stack_p_offset += 1;
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
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
                    .push(format!("push {}", self.get_display_asm(&cloned_lhs)));
                self.stack_p_offset += 1;
                self.cgen_expr(*reclhs, recop, *recrhs);
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
                self.number_for_mangling += 2;
            }
            // THE CASE WHERE BOTH ARE RECURSIVE
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
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
                self.number_for_mangling += 2;
            }
        }
    }
    /// if its a num or iden give how to display it deferenecd
    fn get_display_asm(self: &Self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => n.to_owned(),
            Expr::Iden(a) => match self.initalized_local_vars.get(a) {
                None => format!("{}", qword_deref_helper(a.to_owned())),
                Some(num) => format!("qword [rsp + {} * 8]", (self.stack_p_offset - num - 1)),
            },
            _ => unreachable!(),
        }
    }
    /// takes 2 things on the stack. pops them, does an arg and then pushes the result
    fn cgen_for_stack(self: &mut Self, b_op: &BinOp) -> [String; 4] {
        match b_op {
            &BinOp::Add => add_or_sub_op("add"),
            &BinOp::Sub => add_or_sub_op("sub"),
            &BinOp::Gt => crate::eq_op!("jg", self),
            &BinOp::Lt => crate::eq_op!("jl", self),
            &BinOp::Equ => crate::eq_op!("je", self),
            &BinOp::Lte => crate::eq_op!("jle", self),
            &BinOp::Gte => crate::eq_op!("jge", self),
        }
    }
}

// inline cuz y not
#[inline]
fn add_or_sub_op(op: &str) -> [String; 4] {
    [
        String::from("pop r8"),
        String::from("pop r9"),
        format!("{} r9, r8", op),
        String::from("push r9"),
    ]
}

#[inline]
fn get_op_of_eq_op(jump_cond: &str) -> &str {
    match jump_cond {
        "jl" => "jge",
        "jg" => "jle",
        "jle" => "jg",
        "jge" => "jl",
        "je" => "jne",
        "jn" => "je",
        _ => unreachable!(),
    }
}

#[macro_export]
macro_rules! eq_op {
    ($op:literal, $self:ident) => {{
        $self.stack_p_offset -=1;
        $self.number_for_mangling += 2;
        [
            String::from("pop r8"),
            String::from("pop r9"),
            format!(
                "cmp r9, r8\n{} .IF_HEADER_{}\n{} .IF_HEADER_FAILED_{}",
                $op,
                $self.number_for_mangling,
                get_op_of_eq_op($op),
                $self.number_for_mangling
            ),
            format!(
                ".IF_HEADER_{}\npush 1\njmp .END_IF_HEADER_{}\n.IF_HEADER_FAILED_{}\npush 0\n.END_IF_HEADER_{}",
                $self.number_for_mangling, $self.number_for_mangling, $self.number_for_mangling, $self.number_for_mangling
            ),
        ]
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn codegen_set_stmt() {
        use crate::analyze;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . set   test to 445235 .";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
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
    fn codegen_if_stmt() {
        use crate::analyze;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input =
            "Set y to 5. Set x to (y+5 - 10)+y-15. set z to x + 4. set res_of_bop to x - z < 10.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
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
add r9, r8
push r9
push 10
pop r8
pop r9
sub r9, r8
push r9
push qword [_y]
pop r8
pop r9
add r9, r8
push r9
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
add r9, r8
push r9
pop r8
mov qword [_z], r8
push qword [_x]
push qword [_z]
pop r8
pop r9
sub r9, r8
push r9
push 10
pop r8
pop r9
cmp r9, r8
jl .IF_HEADER_8
jge .IF_HEADER_FAILED_8
.IF_HEADER_8
push 1
jmp .END_IF_HEADER_8
.IF_HEADER_FAILED_8
push 0
.END_IF_HEADER_8
pop r8
mov qword [_res_of_bop], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
_y resq 1
_x resq 1
_z resq 1
_res_of_bop resq 1
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
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyze::analize(&mut ast).unwrap();
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
    fn codegen_loop() {
        use crate::analyze;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "set x to 0. loop, change x to x + 1. if x > 10, break.!!";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        // TODO spelling. one is spelled with y and other with i
        analyze::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        analyze::analize(&mut ast).unwrap();
        code.codegen(ast);
        let correct_code = "global _start
section .text
_start:
mov qword [_x], 0
.START_LOOP_0
push qword [_x]
push 1
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [_x], r8
push qword [_x]
push 10
pop r8
pop r9
cmp r9, r8
jg .IF_HEADER_3
jle .IF_HEADER_FAILED_3
.IF_HEADER_3
push 1
jmp .END_IF_HEADER_3
.IF_HEADER_FAILED_3
push 0
.END_IF_HEADER_3
pop r8
cmp r8, 1
je .IF_BODY_1
jne .IF_END_1
.IF_BODY_1
sub rsp, 0 * 8
jmp .END_LOOP_0
add rsp, 0 * 8
.IF_END_1
jmp .START_LOOP_0
.END_LOOP_0
mov rax, 60
xor rdi, rdi
syscall
section .bss
_x resq 1
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
        let input =
            "Set y to 5. Set x to (y+5 - 10)+y-15. set z to x + 4. set res_of_bop to x - z < 10.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        // TODO spelling. one is spelled with y and other with i
        analyze::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        analyze::analize(&mut ast).unwrap();
        code.codegen(ast);
        let correct_code = "global _start
section .text
_start:
mov qword [_y], 5
push qword [_y]
push 5
pop r8
pop r9
add r9, r8
push r9
push 10
pop r8
pop r9
sub r9, r8
push r9
push qword [_y]
pop r8
pop r9
add r9, r8
push r9
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
add r9, r8
push r9
pop r8
mov qword [_z], r8
push qword [_x]
push qword [_z]
pop r8
pop r9
sub r9, r8
push r9
push 10
pop r8
pop r9
cmp r9, r8
jl .IF_HEADER_8
jge .IF_HEADER_FAILED_8
.IF_HEADER_8
push 1
jmp .END_IF_HEADER_8
.IF_HEADER_FAILED_8
push 0
.END_IF_HEADER_8
pop r8
mov qword [_res_of_bop], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
_y resq 1
_x resq 1
_z resq 1
_res_of_bop resq 1
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
