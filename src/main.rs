#[macro_use]
extern crate lazy_static;

mod token;
mod scanner;
mod lox;
mod util;
mod error_reporter;
mod expression;
mod parser;
mod interpreter;
mod statement;
mod environment;
mod callable;
mod lox_class;

use std::env;
use std::process;

use lox::Lox;
use util::EX_USAGE;

fn main() {
    let mut argv = env::args().skip(1);
    let mut lox = Lox::new();
    if let Some(argument) = argv.next() {
        if let Some(_) = argv.next() {
            println!("Usage: lox-rs [script]");
            process::exit(EX_USAGE);
        } else {
            lox.run_file(&argument);
        }
    } else {
        lox.run_prompt();
    }
}

