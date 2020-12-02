# ezc

![https://github.com/g-w1/ezc/workflows/Rust/badge.svg](https://github.com/g-w1/ezc/workflows/Rust/badge.svg)

Compiler for `ez`.

This is for an independent study in school.

## Goals

- To learn a _lot_
- `ez` should resemble english
- Use only rust standard library
- Write a (minimal) working compiler. probably only u64 but maybe arrays
- Write a (minimal) standard library for `ez` in another language (zig, rust, c, asm, ..)

## Instructions

To run just do `ezc file` use `-g` flag for debug info (it will generate a out.asm file). then you can open in gdb or lldb

To make use of the standard library, pass `-stdlib-path /path/to/stdlib` to the compiler. To compile the standard library, go into the lib directory in this compiler. Then run `zig build` in that directory and find the library in `zig-cache/lib/libstd.a`. You will probably need zig 0.7.1. You can find that here: https://ziglang.org/download/.

To test the code: `cargo test`
To test the generated code you can do `cd tests; ./test.sh`. Note: this requires `gcc`.

## Dependencies:

- Zig (0.7.1). NOTE: If you want to build without zig run `HAS_NO_ZIG=1 cargo build`. This may be useful if you are on a system without zig or want to provide your own standard library.
- `nasm`
- `cargo`
- `gcc` (only for testing)

## Resources

- http://www.eis.mdx.ac.uk/staffpages/r_bornat/books/compiling.pdf - book about compilers
- http://www.egr.unlv.edu/~ed/assembly64.pdf - book about x86-64 assembly
- https://ruslanspivak.com/lsbasi-part1/ - this blog series as a reference to the frontend of a compiler
- [godbolt.org](https://godbolt.org) - an interactive webpage to explore how compilers work on the backend
- http://tinf2.vub.ac.be/~dvermeir/courses/compilers/compilers.pdf - book about compilers.
- https://github.com/ziglang/zig - source code for another programming language
- http://www.cs.ecu.edu/karl/5220/spr16/Notes/Lexical/finitestate.html - explanation of a lexer as a state machine
- https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html - another tutorial about compilers. it is by the leader in the industry compiler toolchain (llvm)
- http://www.cs.man.ac.uk/~pjj/cs2111/ho/node10.html - stack based code generation for expressions
- https://craftinginterpreters.com/ - book about interpreters. could be useful
- https://wiki.osdev.org/System_V_ABI talks about this systemv abi: used for calling conventions. this also does https://wiki.osdev.org/Calling_Conventions

## Features

- [x] lexer

- [x] ast (structs for ast items)

- [x] parser

- [x] codegen

- [x] immutable assignments

- [x] mutable variables

- [x] semantic analysis.

- [x] expressions (the start of recursive parsing)

- [x] if statements

- [x] loops

- [x] fancy compile errors (with carets)

- [x] functions

- [x] modules

- [x] char literals

- [x] standard library

- [x] io

- [x] arrays and string literals

- [x] finish blog bost

- [x] finish documentation
