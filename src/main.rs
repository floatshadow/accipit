use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;
use ariadne::*;
use chumsky::prelude::*;

use accipit::{
    frontend::{
        token::Tokens,
        new_lexer::lexer,
    },
    ir::{
        builders::IRBuilder,
        structures::*
    },
    apps::{
        executor::*,
    }
};


fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("expect input file");
    let src = std::fs::read_to_string(&filename)
        .expect("failed to read input file");

    let (tokens, lex_errs) = lexer().parse(src.as_str()).into_output_errors();
    println!("{:?}", tokens);
    // let (_, tokens) = Lexer::lex(&src)
    //     .finish()
    //     .inspect_err(| lex_err | {
    //         println!("Unrecognized token:\n{}", error::convert_error(src.as_str(), lex_err.clone())) 
    //     })
    //     .map_err(| err | ExecutionError::LexerError ).unwrap();
    // let token_wrapper = Tokens::new(&tokens);
    // let builder = Rc::new(RefCell::new(IRBuilder::new()));
    // let (_, module) = Parser::parse_from_complete_input(token_wrapper, builder).unwrap();
    // println!("Module:\n{}", module);

    // let mut prog_env = ProgramEnv::new();
    // let args = vec![Val::Integer(1), Val::Integer(2)];
    // let interpreted = run_on_module(&mut prog_env, &module, "add_but_direct_link_bb", args);
    // println!("Interepted: {:?}", interpreted);
    // Ok(())

    lex_errs.into_iter()
        .map(| e | e.map_token(| c | c.to_string()))
        .for_each(| e | {
            Report::build(ReportKind::Error, filename.clone(), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new((filename.clone(), e.span().into_range()))
                        .with_message(e.to_string())
                        .with_color(Color::Red)
                )
                .finish()
                .print(sources([(filename.clone(), src.clone())]))
                .unwrap()
        });
}
