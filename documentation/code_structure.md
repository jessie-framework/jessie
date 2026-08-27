# Code structure

The project mainly revolves around the `crates/` directory, where there are multiple Rust crates, and the `tests/` directory, where the compiler is tested.

## The crate directory

These are the crates available at the `crates/` directory as of the moment:

- `jessie` : A CLI tool for compiling/building a Jessie project 
- `jessie-test`: The test runner. It is a binary crate that runs all of the tests in the project.
- `jessie-ast` : Abstract syntax tree (AST) library for the Jessie DSL.
- `jessie-hir` : High level intermediate representation (HIR) of the Jessie DSL. It has some info resolved about various items.
- `jessie-ast-lowering` : The crate responsible for converting `jessie-ast` into `jessie-hir`.
- `jessie-cratesio` : A utility crate responsible for downloading dependencies from [crates.io](https://www.crates.io) in a temporary `.jessie` directory so the compiler can later parse Jessie related dependencies (which by design depend on Jessie)
- `jessie-macros` : Macros for the Jessie DSL. Currently untouched.
- `jessie-rust-lex` : A Rust lexer implementation following (The Rust Reference)[https://doc.rust-lang.org/reference/introduction.html]. We are planning to parse `rustc`'s AST and HIR using this.
- `jessie-session` : A crate holding the compilers session, the session contains a source map and is used for error reporting.
- `jessie-span` : A crate that defines the source map, source map is a data structure where we collect all file names and their source code and we reference into them using `Span`s, which are a range in the source map.
- `jessie-tokenizer` : A lexer for the Jessie DSL. We use two lexers because some tokens (e.g. whitespace) have a meaning in the Jessie DSL but not necessarily for Rust. Do keep in mind that we will also parse `rustc`'s AST and HIR output using `jessie-rust-lex` in the future.

## The tests directory

The tests are run using the `x` utility. To run tests, simply type:

```sh
  ./x test
```

This will run all the available tests. A test file usually consists of a programming language file (`.rs`,`.jessie`) and a `.stderr` file, a file containing a snapshot of the output from the compiler. If the compiler outputs a different output than what is contained in the `.stderr` file, then that means a failed test. Sometimes you may want to update/write new tests. Please look into `writing_tests.md` to learn more.



