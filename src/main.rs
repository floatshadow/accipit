use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;

use accipit::{
    frontend::{
        lexer::Lexer,
        parser::Parser,
        token::Tokens
    },
    ir::{
        builders::IRBuilder,
        structures::*
    }
};


fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("expect input file");
    let src = std::fs::read_to_string(&filename)
        .expect("failed to read input file");

    let (_, tokens) = Lexer::lex(&src).unwrap();
    let token_wrapper = Tokens::new(&tokens);
    let builder = Rc::new(RefCell::new(IRBuilder::new()));
    let (_, module) = Parser::parse_from_complete_input(token_wrapper, builder).unwrap();
    println!("Module:\n{}", module);
}
