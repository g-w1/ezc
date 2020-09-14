# ezc

Compiler for `ez`.

This is for an independent study in school.

## Goals
- To learn a *lot*
- `ez` should resemble english
- Use only rust standard library
- Write a (minimal) working compiler. probably only u64 but maybe arrays
- Write a (minimal) standard library for `ez` in another language (zig, rust, c, asm, ..)

## Instructions

To run (and open in gdb. only works on linux x86_64. To edit input code change `main.rs`): `cargo run -q > out.asm; yasm -g dwarf2 -f elf64 out.asm&&ld out.o -o out && gdb ./out`


To test: `cargo test`


## Resources

* http://www.eis.mdx.ac.uk/staffpages/r_bornat/books/compiling.pdf - book about compilers
* http://www.egr.unlv.edu/~ed/assembly64.pdf - book about x86-64 assembly
* https://ruslanspivak.com/lsbasi-part1/ - this blog series as a reference to the frontend of a compiler
* godbolt.org - an interactive webpage to explore how compilers work on the backend
* http://tinf2.vub.ac.be/~dvermeir/courses/compilers/compilers.pdf - book about compilers.
* https://github.com/ziglang/zig - source code for another programming language
* http://www.cs.ecu.edu/karl/5220/spr16/Notes/Lexical/finitestate.html - explanation of a lexer as a state machine
* https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html - another tutorial about compilers. it is by the leader in the industry compiler toolchain (llvm)
* http://www.cs.man.ac.uk/~pjj/cs2111/ho/node10.html - stack based code generation for expressions

## Features (I may not impliment all of them)

- [x] lexer

- [x] ast (structs for ast items)

- [x] parser

- [x] codegen

- [x] immutable assignments

- [x] mutable variables

- [x] semantic analysis.

- [x] expressions (the start of recursive parsing)

- [ ] control flow

- [ ] functions

- [ ] modules (should be easy since using c abi calling convention)

- [ ] standard library

- [ ] io
