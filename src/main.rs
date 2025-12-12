use crate::{interpreter::interpret, parser::Program};
use std::fs;

mod interpreter;
mod jit;
mod parser;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let source = fs::read_to_string(&args[1]).unwrap();

    let program = Program::from_source(source);
    interpret(&program);
}
