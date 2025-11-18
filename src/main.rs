use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use std::env;
use std::fs::{self, OpenOptions};
use std::io;

pub mod ast;
use jlisp::ast::{Env, Expr};
use jlisp::grammar;

fn setup_builtins() -> Env {
    let env = Env::new();

    // Insert builtin functions
    env.insert("+".to_string(), Expr::Builtin("+".to_string()));
    env.insert("-".to_string(), Expr::Builtin("-".to_string()));
    env.insert("*".to_string(), Expr::Builtin("*".to_string()));
    env.insert("/".to_string(), Expr::Builtin("/".to_string()));

    env.insert("head".to_string(), Expr::Builtin("head".to_string()));
    env.insert("last".to_string(), Expr::Builtin("last".to_string()));
    env.insert("tail".to_string(), Expr::Builtin("tail".to_string()));
    env.insert("list".to_string(), Expr::Builtin("list".to_string()));
    env.insert("join".to_string(), Expr::Builtin("join".to_string()));
    env.insert("eval".to_string(), Expr::Builtin("eval".to_string()));

    env.insert("=".to_string(), Expr::Builtin("=".to_string()));
    env.insert("def".to_string(), Expr::Builtin("def".to_string()));
    env.insert("\\".to_string(), Expr::Builtin("\\".to_string()));

    env
}

fn execute_file(filename: &str) -> io::Result<()> {
    let content = fs::read_to_string(filename)?;
    let env = setup_builtins();
    let pe = grammar::ExprParser::new();

    // Parse and execute each expression in the file
    match pe.parse(&content) {
        Ok(expr) => match expr.eval(env) {
            Ok(result) => println!("{:.?}", result),
            Err(e) => eprintln!("ERROR: {:.?}", e),
        },
        Err(e) => eprintln!("failed to parse file: {:.?}", e),
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // If a file argument is provided, execute it
    if args.len() > 1 {
        let _ = execute_file(&args[1]);
        return Ok(());
    }

    // Otherwise, start the REPL
    let env = setup_builtins();
    let pe = grammar::ExprParser::new();
    let mut rl = DefaultEditor::new()?;

    let repl_hist: String = shellexpand::full("~/.jrepl_hist").unwrap().to_string();
    let _ = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&repl_hist)?;

    rl.load_history(&repl_hist)?;

    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                let expr: Expr = match pe.parse(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("failed to parse line: {:.?}", e);
                        continue;
                    }
                };

                match expr.eval(env) {
                    Ok(v) => println!("{}", v),
                    Err(e) => println!("ERROR: {:.?}", e),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("Goodbye...");
                break;
            }
            Err(err) => {
                println!("Error: {:.?}", err);
                break;
            }
        }
    }
    rl.save_history(&repl_hist)?;
    Ok(())
}
