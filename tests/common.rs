use jlisp::ast::Expr;

pub fn eval_str(input: &str) -> Result<Expr, Box<dyn std::error::Error + '_>> {
    let env = jlisp::builtin::setup_builtins();
    let pe = jlisp::grammar::ExprParser::new();
    let pres = pe.parse(input)?;
    let res = pres.eval(env, 0)?;
    Ok(res)
}
