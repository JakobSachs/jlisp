use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use lalrpop_util::ParseError;

use std::env;
use std::fs::{self, OpenOptions};
use std::io;

pub mod ast;
use jlisp::ast::{Env, Expr, JLisp};
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

fn loc_to_line(src: &str, byte: usize) -> String {
    let mut line = 1;
    for (i, ch) in src.char_indices() {
        if i >= byte {
            break;
        }
        if ch == '\n' {
            line += 1;
        }
    }
    String::from(src.lines().nth(line).unwrap())
}

fn execute_file(filename: &str) -> io::Result<()> {
    let content = fs::read_to_string(filename)?;
    let env = setup_builtins();
    let pe = grammar::JLispParser::new();

    // Parse and execute each expression in the file
    match pe.parse(&content) {
        Ok(jl) => {
            for expr in jl.exprs {
                // in Ok just continue
                if let Err(e) = expr.eval(env,0) {
                    eprintln!("error during eval: {}", e);
                    break;
                }
            }
        }
        Err(e) => match e {
            ParseError::InvalidToken { location } => {
                eprintln!(
                    "invalid token: '{}' in line: {}",
                    &content[location - 5..location + 5],
                    loc_to_line(&content, location)
                )
            }
            ParseError::UnrecognizedEof {
                location: _,
                expected,
            } => eprintln!("EOF expected: {:.?}", expected),
            ParseError::UnrecognizedToken { token, expected } => eprintln!(
                "unexpected input: '{}' in line: {}\n\texpected: {:.?}",
                &content[(token.0)..(token.2)],
                loc_to_line(&content, token.0),
                expected
            ),

            ParseError::ExtraToken { token } => eprintln!(
                "unexpected extra input: '{}'",
                &content[(token.0)..(token.2)]
            ),
            _ => todo!(),
        },
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // file argument provided, execute it
    if args.len() > 1 {
        let _ = execute_file(&args[1]);
        return Ok(());
    }

    //  start the REPL
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
                        println!("failed to parse line: {}\n\tError: {:.?}", line, e);
                        continue;
                    }
                };

                match expr.eval(env,0) {
                    Ok(v) => println!("{}", v),
                    Err(e) => println!("ERROR: {}", e),
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
