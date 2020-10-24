//! code generation for the compiler

use crate::ast::{AstNode, AstRoot, BinOp, Expr, Val};
use std::collections::HashMap;
use std::collections::HashSet;
const FUNCTION_PARAMS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
/// section .bss
#[derive(Debug)]
pub struct Bss {
    pub instructions: Vec<String>,
}

/// section .text
#[derive(Debug)]
pub struct Text {
    pub instructions: Vec<String>,
    pub function_names: Vec<String>,
    pub external_function_names: Vec<String>,
}

/// represent asm
#[derive(Debug)]
pub struct Code {
    bss: Bss,
    text: Text,
    initalized_local_vars: HashMap<String, (u32, bool)>, // the bool is wether it is an array or not
    initalized_static_vars: HashMap<String, bool>,
    initalized_array_lengths: HashMap<String, u32>,
    number_for_mangling: u32,
    stack_p_offset: u32,
    cur_func: String,
}

impl Code {
    pub fn new() -> Self {
        Code {
            text: Text {
                instructions: Vec::new(),
                function_names: Vec::new(),
                external_function_names: Vec::new(),
            },
            bss: Bss {
                instructions: Vec::new(),
            },
            number_for_mangling: 0,
            stack_p_offset: 0,
            initalized_local_vars: HashMap::new(),
            initalized_static_vars: HashMap::new(),
            initalized_array_lengths: HashMap::new(),
            cur_func: String::new(),
        }
    }
    /// generate the code. dont deal with any of the sections
    pub fn cgen(&mut self, tree: AstRoot) {
        for var in tree.static_vars.unwrap() {
            self.bss.instructions.push(format!(
                "MaNgLe_{} resq {}",
                &var.0,
                match var.1 {
                    crate::analyse::Type::Arr(n) => {
                        self.initalized_static_vars.insert(var.0.clone(), true);
                        n + 1 // + 1 because 1st elem in array is len
                    }
                    crate::analyse::Type::Number => {
                        self.initalized_static_vars.insert(var.0.clone(), false);
                        1
                    } // if its a number we just allocate 1 byte
                }
            ));
        }
        let mut started_tl = false;
        for node in tree.tree {
            if let AstNode::Func { .. } | AstNode::Extern { .. } = node {
            } else {
                if !started_tl {
                    self.text.instructions.push(String::from("_start:")); // TODO change if going recursive
                    started_tl = true;
                }
            }
            match node {
                AstNode::SetOrChange { sete, setor, .. } => {
                    self.cgen_set_or_change_stmt(sete, setor)
                }
                AstNode::If {
                    guard,
                    body,
                    vars_declared,
                } => self.cgen_if_stmt(guard, vars_declared.unwrap(), body, None), // we unwrap because it was analised
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                AstNode::Func {
                    name,
                    args,
                    body,
                    vars_declared,
                    export,
                } => self.cgen_function(name, args, body, vars_declared.unwrap(), export),
                AstNode::Extern { name, .. } => self.text.external_function_names.push(name),
                _ => unreachable!(),
            }
        }
        if !started_tl {
            self.text.instructions.push(String::from("_start:"))
        }
    }
    fn cgen_return_stmt(&mut self, val: Expr) {
        self.cgen_expr(val);
        self.text
            .instructions
            .push(format!("mov rax, r8\njmp .RETURN_{}", self.cur_func));
    }
    /// a little helper fn
    fn reg_to_farness_stack(&mut self, n: usize) -> i8 {
        if n < 6 {
            self.text
                .instructions
                .push(format!("push {}", FUNCTION_PARAMS[n]));
            self.stack_p_offset += 1;
            n as i8
        } else {
            unimplemented!();
            // // TODO this work
            // self.text
            //     .instructions
            //     .push(format!("push qword [rsp - {}]", 8 * (n + 7)));
            // self.stack_p_offset += 1;
            // n as i8
        }
    }
    // ////////////////////////////////////////////////////////////  Systemv abi: https://wiki.osdev.org/Calling_Conventions
    // Platform | Return Value | Parameter Registers        | Additional Parameters |Stack Alignment | Scratch Registers 	                     | Preserved Registers 	             | Call List
    // 86_64    | rax, rdx 	   | rdi, rsi, rdx, rcx, r8, r9 | stack (right to left) |16-byte at call | rax, rdi, rsi, rdx, rcx, r8, r9, r10, r11 | rbx, rsp, rbp, r12, r13, r14, r15 | rbp
    /// codegen for a function decl
    fn cgen_function(
        &mut self,
        name: String,
        args: Vec<crate::ast::Type>,
        body: Vec<AstNode>,
        vars_declared: HashMap<String, (u32, bool)>,
        export: bool,
    ) {
        /////////////////////////// Some setup ///////////////////////// clear local vars bc a func starts with none
        self.initalized_local_vars.clear();
        if !export {
            self.text.function_names.push(format!("MaNgLe_{}", &name)); // declaring it global
            self.text.instructions.push(format!("MaNgLe_{}:", &name));
        } else {
            self.text.function_names.push(format!("{}", &name)); // declaring it global
            self.text.instructions.push(format!("{}:", &name));
        }
        self.cur_func = name; //doing the args
                              ////
                              ////////// SETUP STACK //////////////////////
                              ////
        self.text
            .instructions
            .push(String::from("push rbp\nmov rbp, rsp"));
        self.stack_p_offset += 1;
        let (double_keys, mem_len) = self.cgen_setup_stack(&vars_declared, Some(&args));
        ////
        //////////////////// BODY /////////////////////
        ////
        for node in body {
            match node {
                AstNode::SetOrChange { sete, setor, .. } => {
                    self.cgen_set_or_change_stmt(sete, setor)
                }
                AstNode::If {
                    body,
                    guard,
                    vars_declared,
                } => self.cgen_if_stmt(guard, vars_declared.unwrap(), body, None),
                AstNode::Return { val } => self.cgen_return_stmt(val),
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                _ => unreachable!(), // function or break statement
            }
        }
        ////
        //////////////////////////////////////  UNSETUP STACK ////////////////////////////////////////
        ////
        self.text.instructions.push(String::from("mov rax, 0")); //return 0 if havent returned b4
        self.text
            .instructions
            .push(format!(".RETURN_{}", self.cur_func));
        for (var, _) in &vars_declared {
            if !double_keys.contains(var) {
                self.initalized_local_vars.remove(var);
            }
        }
        self.text.instructions.push(String::from("mov rsp, rbp"));
        self.stack_p_offset -= mem_len as u32;
        self.text.instructions.push(String::from("pop rbp"));
        self.stack_p_offset -= 1;
        self.text.instructions.push(String::from("ret"));
        ////////////////// cleanup ///////////////
        self.initalized_local_vars.clear(); // clear initalized vars
    }
    /// code generation for a function call
    fn cgen_funcall_expr(&mut self, func_name: &str, mangle: bool, args: &Vec<Val>) {
        for (i, arg) in args.iter().enumerate() {
            if i > 6 {
                unimplemented!()
            }
            match arg {
                Val::Expr(e) => self.cgen_expr(e.clone()),
                // TODO do this. lol
                Val::Array(_ve) => self.text.instructions.push(format!("mov r8, {}", { "a" })),
            }
            self.text
                .instructions
                .push(format!("mov {}, r8", FUNCTION_PARAMS[i]));
        }
        if !mangle {
            self.text
                .instructions
                .push(format!("call MaNgLe_{}\nmov r8, rax", func_name));
        } else {
            self.text
                .instructions
                .push(format!("call {}\nmov r8, rax", func_name));
        }
    }
    /// code generation for a val. []n or n
    fn cgen_array_set_or_change(&mut self, ve: Vec<Expr>, sete: &str) {
        // TODO may need to reverse array. need to know how its layed out in memory
        // check COMPILER EXPLORER
        // we do one more than the array because the first elem in the array is the len of it.
        // TODO need to use bss for this not stack if static
        // not sure if this is a bad decision
        let len_of_arr = ve.len() as u32;
        if let Some(off) = self.initalized_local_vars.get(sete) {
            // we know it is a stack allocated var
            // move the length to the first element in the array
            self.text
                .instructions
                .push(format!("mov r8, {}", len_of_arr));
            self.text.instructions.push(format!(
                "mov [rsp + {} * 8], r8",
                (self.stack_p_offset + off.0 - 1 - len_of_arr),
            ));
            let newoff = off.clone(); // we do this to avoid weird ownership stuff. not my proudest code
            for (i, e) in ve.iter().rev().enumerate() {
                self.cgen_expr(e.clone());
                let tmpval = self.stack_p_offset - &newoff.0 - i as u32 - 1;
                self.text
                    .instructions
                    .push(format!("mov [rsp + {} * 8 ], r8", tmpval));
            }
        } else {
            // move the length to the first element in the array
            self.text
                .instructions
                .push(format!("mov qword MaNgLe_{}[0], {}", sete, len_of_arr));
            // we know it is a static var
            for (i, e) in ve.iter().enumerate() {
                self.cgen_expr(e.clone()); // TODO get rid of this clone
                self.text.instructions.push(format!(
                    "mov qword MaNgLe_{}[{} * 8], r8",
                    sete,
                    i + 1
                ));
                // TODO we need ptr here?
            }
        }
    }
    /// code generation for an expr. moves the result to r8
    fn cgen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::BinOp { lhs, op, rhs } => {
                self.cgen_binop_expr(*lhs, op, *rhs);
                self.text.instructions.push(String::from("pop r8"));
                self.stack_p_offset -= 1;
            }
            Expr::Number(n) => self.text.instructions.push(format!("mov r8, {}", n)), // TODO really easy optimisation by just parsing num at compile time. but right now this is easier. premature optimisation is the start of all evil
            Expr::Iden(i) => {
                let tmpexpr = self.get_display_asm(&Expr::Iden(i.clone()));
                if self.initalized_local_vars.get(&i).is_some() {
                    self.text.instructions.push(format!("lea r8, {}", tmpexpr));
                } else {
                    self.text.instructions.push(format!("mov r8, {}", tmpexpr));
                }
            }

            Expr::FuncCall {
                func_name,
                args,
                external,
            } => self.cgen_funcall_expr(&func_name, external.unwrap(), &args),
        }
    }
    fn cgen_setup_stack(
        &mut self,
        vars_declared: &HashMap<String, (u32, bool)>,
        args: Option<&Vec<crate::ast::Type>>,
    ) -> (HashSet<String>, u32) {
        let mut double_keys: HashSet<String> = HashSet::new();
        // the use of double_keys is something like this. when something is declared inside a block and also needs to be out of the block. cant drop it. but do drop the memory. just not drop it from the initial hashmap:
        // set z to 5. if z = 5,
        //     set a to 5.
        //     if a > 4,
        //         set test to a.
        //     !
        //     set test to 5.
        // !
        let mem_len = {
            let mut max = 0;
            for (_, (n, _)) in vars_declared {
                max += n;
            }
            max
        };
        self.stack_p_offset += mem_len as u32;
        self.text
            .instructions
            .push(format!("sub rsp, {} * 8", mem_len)); // allocate locals
        if let Some(x) = args {
            let mut tmp;
            for (i, arg) in x.iter().enumerate() {
                tmp = self.stack_p_offset - self.reg_to_farness_stack(i) as u32 + i as u32;
                let mut isarray: bool = false;
                let name = match arg {
                    crate::ast::Type::Num(s) => s,
                    crate::ast::Type::ArrNum(s, len_of_arr) => {
                        isarray = true;
                        self.initalized_array_lengths
                            .insert(s.clone(), len_of_arr.parse().unwrap());
                        s
                    }
                }; // arrays are passed by reference so we only need to incriment stack pointer by 1 still
                self.initalized_local_vars
                    .insert(name.clone(), (tmp, isarray));
            }
        }
        for (varname, place) in vars_declared.to_owned() {
            if self.initalized_local_vars.contains_key(&varname) {
                double_keys.insert(varname.clone());
            }
            self.initalized_local_vars.insert(
                varname.clone(),
                (self.stack_p_offset as u32 - place.0, place.1),
            );
            if place.1 {
                self.initalized_array_lengths.insert(varname, place.0);
            }
        }
        (double_keys, mem_len)
    }
    /// code gen for if stmt. uses stack based allocation
    fn cgen_if_stmt(
        &mut self,
        guard: Expr,
        vars: HashMap<String, (u32, bool)>,
        body: Vec<AstNode>,
        loop_num: Option<u32>,
    ) {
        ///////////////////////////////////// EVALUATE THE ACTUAL BOOL //////////////////////////////
        let our_number_for_mangling = self.number_for_mangling;
        self.cgen_expr(guard);
        self.text.instructions.push(String::from("cmp r8, 1"));
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
        let (double_keys, mem_len) = self.cgen_setup_stack(&vars, None);
        for node in body {
            match node {
                AstNode::Func { .. } => unreachable!(),
                AstNode::Return { val } => self.cgen_return_stmt(val),
                AstNode::If {
                    body,
                    guard,
                    vars_declared,
                } => self.cgen_if_stmt(guard, vars_declared.unwrap(), body, None),
                AstNode::SetOrChange { sete, setor, .. } => {
                    self.cgen_set_or_change_stmt(sete, setor)
                }
                AstNode::Loop { body } => self.cgen_loop_stmt(body),
                AstNode::Break => self
                    .text
                    .instructions
                    .push(format!("jmp .END_LOOP_{}", loop_num.unwrap())),
                _ => unreachable!(),
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
        self.stack_p_offset -= mem_len as u32;
        self.text
            .instructions
            .push(format!(".IF_END_{}", our_number_for_mangling));
        self.number_for_mangling += 1;
    }
    /// code generation for a loop. very easy
    fn cgen_loop_stmt(&mut self, body: Vec<AstNode>) {
        let our_number_for_mangling = self.number_for_mangling;
        self.number_for_mangling += 1;
        self.text
            .instructions
            .push(format!(".START_LOOP_{}", our_number_for_mangling));
        for node in body {
            match node {
                AstNode::Return { val } => self.cgen_return_stmt(val),
                AstNode::Func { .. } => unreachable!(),
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
                AstNode::SetOrChange { change: false, .. } => unreachable!(),
                _ => unreachable!(),
            }
        }
        self.text.instructions.push(format!(
            "jmp .START_LOOP_{}\n.END_LOOP_{}",
            our_number_for_mangling, our_number_for_mangling
        ))
    }
    /// code generation for a set or change stmt. it is interpreted as change if change is true
    fn cgen_set_or_change_stmt(&mut self, sete: String, setor: Val) {
        match setor {
            Val::Expr(e) => {
                self.cgen_expr(e);
                let tmpsete = self.get_display_asm(&Expr::Iden(sete));
                self.text.instructions.push(format!("mov {}, r8", tmpsete,));
            }
            Val::Array(ae) => {
                self.cgen_array_set_or_change(ae, &sete);
            }
        }
    }

    /// A function to recursively generate code for expressions.
    fn cgen_binop_expr(&mut self, lhs: Expr, op: BinOp, rhs: Expr) {
        let cloned_rhs = rhs.clone();
        let cloned_lhs = lhs.clone();
        match (lhs, rhs) {
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
                self.cgen_binop_expr(*lreclhs, lrecop, *lrecrhs);
                self.number_for_mangling += 1;
                self.cgen_binop_expr(*rreclhs, rrecop, *rrecrhs);
                self.number_for_mangling += 1;
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
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
                _,
            ) => {
                self.cgen_binop_expr(*reclhs, recop, *recrhs);
                let tmprhs = self.get_display_asm(&cloned_rhs);
                self.text.instructions.push(format!("push {}", tmprhs));
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
                _,
                Expr::BinOp {
                    lhs: reclhs,
                    op: recop,
                    rhs: recrhs,
                },
            ) => {
                let tmplhs = self.get_display_asm(&cloned_lhs);
                self.text.instructions.push(format!("push {}", tmplhs));
                self.stack_p_offset += 1;
                self.cgen_binop_expr(*reclhs, recop, *recrhs);
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
                self.number_for_mangling += 2;
            }
            // base case
            //       op
            //     /    \
            //  num     num
            (_, _) => {
                let tmplhs = self.get_display_asm(&cloned_lhs);
                self.text.instructions.push(format!("push {}", tmplhs));
                self.stack_p_offset += 1;
                let tmprhs = self.get_display_asm(&cloned_rhs);
                self.text.instructions.push(format!("push {}", tmprhs));
                self.stack_p_offset += 1;
                let slice = &self.cgen_for_stack(&op);
                self.text.instructions.extend_from_slice(slice);
            }
        }
    }
    /// if its a num or iden give how to display it deferenecd
    fn get_display_asm(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => n.to_owned(),
            Expr::Iden(a) => match self.initalized_local_vars.get(a) {
                None => {
                    if !self.initalized_static_vars.get(a).unwrap() {
                        format!("qword [MaNgLe_{}]", a)
                    } else {
                        format!("MaNgLe_{}", a)
                    }
                }
                Some(num) => {
                    let val = if let Some(z) = self.initalized_array_lengths.get(a) {
                        self.stack_p_offset + num.0 - z
                    } else {
                        self.stack_p_offset - num.0 - 1
                    };
                    format!("qword [rsp + {} * 8]", val)
                }
            },
            Expr::FuncCall {
                func_name,
                args,
                external,
            } => {
                self.cgen_funcall_expr(&func_name, external.unwrap(), args);
                format!("r8")
            }
            _ => unreachable!(),
        }
    }
    /// takes 2 things on the stack. pops them, does an arg and then pushes the result
    fn cgen_for_stack(&mut self, b_op: &BinOp) -> [String; 4] {
        match b_op {
            &BinOp::Add => self.special_bop("add"),
            &BinOp::Sub => self.special_bop("sub"),
            &BinOp::Mul => self.special_bop("imul"),
            &BinOp::Or => self.special_bop("or"),
            &BinOp::And => self.special_bop("and"),
            &BinOp::Gt => crate::eq_op!("jg", self),
            &BinOp::Gte => crate::eq_op!("jge", self),
            &BinOp::Lt => crate::eq_op!("jl", self),
            &BinOp::Equ => crate::eq_op!("je", self),
            &BinOp::Ne => crate::eq_op!("jne", self),
            &BinOp::Lte => crate::eq_op!("jle", self),
        }
    }
    // inline cuz y not
    // TODO test if this works
    #[inline]
    fn special_bop(&mut self, op: &str) -> [String; 4] {
        self.stack_p_offset -= 1;
        [
            String::from("pop r8"),
            String::from("pop r9"),
            format!("{} r9, r8", op),
            String::from("push r9"),
        ]
    }
}

#[inline]
fn get_op_of_eq_op(jump_cond: &str) -> &str {
    match jump_cond {
        "jl" => "jge",
        "jg" => "jle",
        "jle" => "jg",
        "jge" => "jl",
        "je" => "jne",
        "jne" => "je",
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
impl Code {
    /// printing the asm to stdout. should be very easy to port to file because stdout is a file!!!
    pub fn fmt(&self, lib: bool) -> String {
        // adding the sections
        let mut f = String::new();
        use std::fmt::Write;
        if self.text.instructions.clone().len() > 0 {
            for i in &self.text.external_function_names {
                writeln!(f, "extern {}", i).unwrap();
            }
            if !lib {
                writeln!(f, "global _start").unwrap();
            }
            for i in &self.text.function_names {
                writeln!(f, "global {}", i).unwrap();
            }
            writeln!(f, "section .text").unwrap();
            if lib {
                for i in self
                    .text
                    .instructions
                    .iter()
                    .take_while(|s| **s != "_start:".to_string())
                {
                    writeln!(f, "{}", i).unwrap();
                }
            } else {
                for i in &self.text.instructions {
                    writeln!(f, "{}", i).unwrap();
                }
            }
            // exit 0
            if !lib {
                writeln!(
                    f,
                    "mov rax, 60
xor rdi, rdi
syscall"
                )
                .unwrap();
            }
        }
        if self.bss.instructions.clone().len() > 0 {
            writeln!(f, "section .bss").unwrap();
            for i in &self.bss.instructions {
                writeln!(f, "{}", i).unwrap();
            }
        }
        f
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn codegen_set_stmt() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . set   test to 445235 .";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 10
mov qword [MaNgLe_x], r8
mov r8, 5
mov qword [MaNgLe_y], r8
mov r8, 445235
mov qword [MaNgLe_test], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_x resq 1
MaNgLe_y resq 1
MaNgLe_test resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_if_stmt1() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . if y != x, change x to y.!";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 10
mov qword [MaNgLe_x], r8
mov r8, 5
mov qword [MaNgLe_y], r8
push qword [MaNgLe_y]
push qword [MaNgLe_x]
pop r8
pop r9
cmp r9, r8
jne .IF_HEADER_2
je .IF_HEADER_FAILED_2
.IF_HEADER_2
push 1
jmp .END_IF_HEADER_2
.IF_HEADER_FAILED_2
push 0
.END_IF_HEADER_2
pop r8
cmp r8, 1
je .IF_BODY_0
jne .IF_END_0
.IF_BODY_0
sub rsp, 0 * 8
mov r8, qword [MaNgLe_y]
mov qword [MaNgLe_x], r8
add rsp, 0 * 8
.IF_END_0
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_x resq 1
MaNgLe_y resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_if_stmt2() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "
set c to 0.

if 5 >= 5,
  set arst to 5.
  set arstarst to 6.
  change arst to 7.
  if arst > arstarst,
    change c to 5.
  !
!
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 0
mov qword [MaNgLe_c], r8
push 5
push 5
pop r8
pop r9
cmp r9, r8
jge .IF_HEADER_2
jl .IF_HEADER_FAILED_2
.IF_HEADER_2
push 1
jmp .END_IF_HEADER_2
.IF_HEADER_FAILED_2
push 0
.END_IF_HEADER_2
pop r8
cmp r8, 1
je .IF_BODY_0
jne .IF_END_0
.IF_BODY_0
sub rsp, 2 * 8
mov r8, 5
mov qword [rsp + 0 * 8], r8
mov r8, 6
mov qword [rsp + 1 * 8], r8
mov r8, 7
mov qword [rsp + 0 * 8], r8
push qword [rsp + 0 * 8]
push qword [rsp + 2 * 8]
pop r8
pop r9
cmp r9, r8
jg .IF_HEADER_4
jle .IF_HEADER_FAILED_4
.IF_HEADER_4
push 1
jmp .END_IF_HEADER_4
.IF_HEADER_FAILED_4
push 0
.END_IF_HEADER_4
pop r8
cmp r8, 1
je .IF_BODY_2
jne .IF_END_2
.IF_BODY_2
sub rsp, 0 * 8
mov r8, 5
mov qword [MaNgLe_c], r8
add rsp, 0 * 8
.IF_END_2
add rsp, 2 * 8
.IF_END_0
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_c resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_expr() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input =
            "Set y to 5. Set x to (y+5 - 10)+y-15. set z to x + 4. set res_of_bop to x - z < 10.";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 5
mov qword [MaNgLe_y], r8
push qword [MaNgLe_y]
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
push qword [MaNgLe_y]
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
mov qword [MaNgLe_x], r8
push qword [MaNgLe_x]
push 4
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [MaNgLe_z], r8
push qword [MaNgLe_x]
push qword [MaNgLe_z]
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
mov qword [MaNgLe_res_of_bop], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_y resq 1
MaNgLe_x resq 1
MaNgLe_z resq 1
MaNgLe_res_of_bop resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_funcall_export() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = " export function AddOne(a),
    return a + 1.
!
set tmp to AddOne(1).
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
global AddOne
section .text
AddOne:
push rbp
mov rbp, rsp
push rdi
sub rsp, 0 * 8
push qword [rsp + 0 * 8]
push 1
pop r8
pop r9
add r9, r8
push r9
pop r8
mov rax, r8
jmp .RETURN_AddOne
mov rax, 0
.RETURN_AddOne
mov rsp, rbp
pop rbp
ret
_start:
mov r8, 1
mov rdi, r8
call AddOne
mov r8, rax
mov qword [MaNgLe_tmp], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_tmp resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_funcall() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Function fib(n),
  if n <= 1,
      return n.
  !
  return fib(n - 1) + fib( n- 2).
!

set z to fib(50).
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
global MaNgLe_fib
section .text
MaNgLe_fib:
push rbp
mov rbp, rsp
push rdi
sub rsp, 0 * 8
push qword [rsp + 0 * 8]
push 1
pop r8
pop r9
cmp r9, r8
jle .IF_HEADER_2
jg .IF_HEADER_FAILED_2
.IF_HEADER_2
push 1
jmp .END_IF_HEADER_2
.IF_HEADER_FAILED_2
push 0
.END_IF_HEADER_2
pop r8
cmp r8, 1
je .IF_BODY_0
jne .IF_END_0
.IF_BODY_0
sub rsp, 0 * 8
mov r8, qword [rsp + 0 * 8]
mov rax, r8
jmp .RETURN_fib
add rsp, 0 * 8
.IF_END_0
push qword [rsp + 0 * 8]
push 1
pop r8
pop r9
sub r9, r8
push r9
pop r8
mov rdi, r8
call MaNgLe_fib
mov r8, rax
push r8
push qword [rsp + 1 * 8]
push 2
pop r8
pop r9
sub r9, r8
push r9
pop r8
mov rdi, r8
call MaNgLe_fib
mov r8, rax
push r8
pop r8
pop r9
add r9, r8
push r9
pop r8
mov rax, r8
jmp .RETURN_fib
mov rax, 0
.RETURN_fib
mov rsp, rbp
pop rbp
ret
_start:
mov r8, 50
mov rdi, r8
call MaNgLe_fib
mov r8, rax
mov qword [MaNgLe_z], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_z resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_change_stmt() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 10. set y to 5 . change   x to 445235 .";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 10
mov qword [MaNgLe_x], r8
mov r8, 5
mov qword [MaNgLe_y], r8
mov r8, 445235
mov qword [MaNgLe_x], r8
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_x resq 1
MaNgLe_y resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_and_bop() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Set x to 1. set y to 0 . if y and x, change x to 10.!";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 1
mov qword [MaNgLe_x], r8
mov r8, 0
mov qword [MaNgLe_y], r8
push qword [MaNgLe_y]
push qword [MaNgLe_x]
pop r8
pop r9
and r9, r8
push r9
pop r8
cmp r8, 1
je .IF_BODY_0
jne .IF_END_0
.IF_BODY_0
sub rsp, 0 * 8
mov r8, 10
mov qword [MaNgLe_x], r8
add rsp, 0 * 8
.IF_END_0
mov rax, 60
xor rdi, rdi
syscall
section .bss
MaNgLe_x resq 1
MaNgLe_y resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
    #[test]
    fn codegen_lib_func() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "Function fib(n),
  set counter to 1.
  set a to 1.
  set b to 0.
  [start the loop]
  loop,
    change a to a + b.
    change counter to counter + 1.
    if counter > n,
      return a.
    !
    change b to a + b.
    change counter to counter + 1.
    if counter > n,
      return b.
    !
  !
!
";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        code.cgen(ast);
        let correct_code = "global MaNgLe_fib
section .text
MaNgLe_fib:
push rbp
mov rbp, rsp
push rdi
sub rsp, 3 * 8
mov r8, 1
mov qword [rsp + 0 * 8], r8
mov r8, 1
mov qword [rsp + 1 * 8], r8
mov r8, 0
mov qword [rsp + 2 * 8], r8
.START_LOOP_0
push qword [rsp + 1 * 8]
push qword [rsp + 3 * 8]
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [rsp + 1 * 8], r8
push qword [rsp + 0 * 8]
push 1
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [rsp + 0 * 8], r8
push qword [rsp + 0 * 8]
push qword [rsp + 4 * 8]
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
mov r8, qword [rsp + 1 * 8]
mov rax, r8
jmp .RETURN_fib
add rsp, 0 * 8
.IF_END_1
push qword [rsp + 1 * 8]
push qword [rsp + 3 * 8]
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [rsp + 2 * 8], r8
push qword [rsp + 0 * 8]
push 1
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [rsp + 0 * 8], r8
push qword [rsp + 0 * 8]
push qword [rsp + 4 * 8]
pop r8
pop r9
cmp r9, r8
jg .IF_HEADER_6
jle .IF_HEADER_FAILED_6
.IF_HEADER_6
push 1
jmp .END_IF_HEADER_6
.IF_HEADER_FAILED_6
push 0
.END_IF_HEADER_6
pop r8
cmp r8, 1
je .IF_BODY_4
jne .IF_END_4
.IF_BODY_4
sub rsp, 0 * 8
mov r8, qword [rsp + 2 * 8]
mov rax, r8
jmp .RETURN_fib
add rsp, 0 * 8
.IF_END_4
jmp .START_LOOP_0
.END_LOOP_0
mov rax, 0
.RETURN_fib
mov rsp, rbp
pop rbp
ret
";
        assert_eq!(format!("{}", code.fmt(true)), correct_code);
    }
    #[test]
    fn codegen_loop() {
        use crate::analyse;
        use crate::codegen;
        use crate::lexer;
        use crate::parser;

        let mut tokenizer = lexer::Tokenizer::new();
        let input = "set x to 0. loop, change x to x + 1. if x > 10, break.!!";
        let output = tokenizer.lex(&String::from(input));
        let mut ast = parser::parse(output.0.unwrap(), output.1).unwrap();
        analyse::analize(&mut ast).unwrap();
        let mut code = codegen::Code::new();
        analyse::analize(&mut ast).unwrap();
        code.cgen(ast);
        let correct_code = "global _start
section .text
_start:
mov r8, 0
mov qword [MaNgLe_x], r8
.START_LOOP_0
push qword [MaNgLe_x]
push 1
pop r8
pop r9
add r9, r8
push r9
pop r8
mov qword [MaNgLe_x], r8
push qword [MaNgLe_x]
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
MaNgLe_x resq 1
";
        assert_eq!(format!("{}", code.fmt(false)), correct_code);
    }
}
