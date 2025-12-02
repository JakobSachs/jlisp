use jlisp::ast::Expr;
mod common;
use common::eval_str;


#[test]
fn test_lambda() {
    let res = eval_str("((\\ [x] (+ x 1)) 2)").unwrap();
    assert_eq!(res, Expr::Number(3));
}
