use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use std::fs::OpenOptions;
use std::rc::Rc;

pub mod ast;
use jlisp::ast::{Env, Expr};
use jlisp::grammar;

macro_rules! insert_builtin {
    ($map:expr, $str:expr) => {
        $map.map
            .borrow_mut()
            .insert($str.to_string(), Expr::Builtin($str.to_string()))
    };
}

fn setup_builtins() -> Rc<Env> {
    let out = Env::new();
    let _ = insert_builtin!(out, "+");
    let _ = insert_builtin!(out, "-");
    let _ = insert_builtin!(out, "*");
    let _ = insert_builtin!(out, "/");

    let _ = insert_builtin!(out, "head");
    let _ = insert_builtin!(out, "last");
    let _ = insert_builtin!(out, "tail");
    let _ = insert_builtin!(out, "list");
    let _ = insert_builtin!(out, "join");
    let _ = insert_builtin!(out, "eval");

    let _ = insert_builtin!(out, "=");
    let _ = insert_builtin!(out, "def");
    let _ = insert_builtin!(out, "\\");

    out
}

fn main() -> Result<()> {
    let env = setup_builtins();

    let repl_hist: String = shellexpand::full("~/.jrepl_hist").unwrap().to_string();
    // let p = grammar::JLispParser::new();
    let pe = grammar::ExprParser::new();

    let mut rl = DefaultEditor::new()?;

    // touch REPL_HIST
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

                match expr.eval(env.clone()) {
                    Ok(v) => println!("{:.?}", v),
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
