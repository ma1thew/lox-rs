# lox-rs

A recursive descent parser and AST walking interpreter for Lox, an object-oriented, dynamically typed language, with closures, first-class functions, and some static analysis. However, this implementation does not include inheritance.
This implementation is based off of the Java implementation in [*Crafting Interpreters*](https://craftinginterpreters.com).
However, there are some significant architectural changes as a result of Rust's not-quite object oriented nature.
Some example Lox code can be found in [the Crafting Intepreters repository](https://github.com/munificent/craftinginterpreters), in the `test/` subdirectory.