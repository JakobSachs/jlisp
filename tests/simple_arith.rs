use jlisp::ast::{Error, Expr};
mod common;
use common::eval_str;

#[test]
fn test_add() {
    let res = eval_str("(+ 1 2)").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_sub() {
    let res = eval_str("(- 10 2)").unwrap();
    assert_eq!(res, Expr::Number(8));
}

#[test]
fn test_mul() {
    let res = eval_str("(* 10 2)").unwrap();
    assert_eq!(res, Expr::Number(20));
}

#[test]
fn test_div() {
    let res = eval_str("(/ 10 2)").unwrap();
    assert_eq!(res, Expr::Number(5));
}

#[test]
fn test_nested_expr() {
    let res = eval_str("(+ 1 (* 2 3))").unwrap();
    assert_eq!(res, Expr::Number(7));
}

#[test]
fn test_div_by_zero() {
    let res = eval_str("(/ 10 0)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::DivisionByZero { .. }));
}
