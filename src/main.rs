#[macro_use]
extern crate failure;
#[macro_use]
extern crate approx;

use std::env;

pub mod lang;
pub mod parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", crate::lang::assemble::assemble(args[1].clone()));
}
