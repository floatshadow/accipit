use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;
use nom::*;
use clap::Parser;

use accipit::{
    frontend::{
        token::{Tokens, Token},
        lexer,
        parser,
    },
    ir::{
        builders::IRBuilder,
        structures::*
    },
    apps::{
        executor::*,
    }
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(name = "accipit")]
#[command(bin_name = "accipit")]
pub struct Args {
    /// Specify the input file
    #[clap(value_parser=clap::value_parser!(PathBuf))]
    file: PathBuf,

    /// Specify the argument passes to the entry function
    #[clap(value_parser=clap::value_parser!(String))]
    args: Vec<String>,

    /// Specify the output file (unused option)
    #[clap(short, long, value_parser=clap::value_parser!(PathBuf))]
    output: Option<PathBuf>,

    /// Dump parsed module, producing explicit type annotation and different symbol prefix
    #[clap(long)]
    dump_module: bool,

    /// Specify the certain function as the entry function
    #[clap(short, long = "entry", default_value = "main")]
    entry: String,
}

fn main() -> Result<(), ()>{
    let args = Args::parse();
    let input = args.file;
    let src = std::fs::read_to_string(input)
        .expect("failed to read input file");

    // let (tokens, lex_errs) = lexer().parse(src.as_str()).into_output_errors();
    // println!("{:?}", tokens);
    let (_, tokens) = lexer::Lexer::lex(&src)
        .finish()
        .inspect_err(| lex_err | {
            println!("Unrecognized token:\n{}", nom::error::convert_error(src.as_str(), lex_err.clone())) 
        })
        .map_err(| _ | () )?;
    // println!("{:?}", tokens);
    let token_wrapper = Tokens::new(&tokens);
    let builder = Rc::new(RefCell::new(IRBuilder::new()));
    let (_, module) = parser::Parser::parse_from_complete_input(token_wrapper, builder)
        .finish()
        .inspect_err(| parser_err | {
            println!("Parser Error:\n{:?}", parser_err) 
        })
        .map_err(| _ | () )?;

    // dump module
    if args.dump_module {
        println!("Module:\n{}", module);
    }

    let mut prog_env = ProgramEnv::new();
    let entry_fn = args.entry;
    let input_args: Vec<Val> = args.args
        .iter()
        .map(| input_str | 
            Val::from_str(input_str)
        )
        .collect::<Result<_, _>>()
        .inspect_err( | err | {
            println!("{}", err)
        })
        .map_err( | _ | ())?;
    let interpreted = run_on_module(&mut prog_env, &module, &entry_fn, input_args)
        .inspect_err( | interpreted_err | {
            println!("{}", interpreted_err);
        })
        .map_err(| _ | ())?;
    println!("{}", interpreted);
    Ok(())

}
