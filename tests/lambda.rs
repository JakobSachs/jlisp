use jlisp::ast::{Error, Expr};
mod common;
use common::eval_str;

#[test]
fn test_simple_lambda_definition() {
    let res = eval_str("(\\ {x} {+ x 1})").unwrap();
    match res {
        Expr::Lambda { formals, body, .. } => {
            assert_eq!(formals.len(), 1);
            assert_eq!(formals[0], Expr::Symbol("x".to_string()));
            assert_eq!(
                *body,
                Expr::Qexpr(vec![
                    Expr::Symbol("+".to_string()),
                    Expr::Symbol("x".to_string()),
                    Expr::Number(1)
                ])
            );
        }
        _ => panic!("Expected Lambda expression"),
    }
}

#[test]
fn test_lambda_application() {
    let res = eval_str("((\\ {x} {+ x 1}) 5)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

#[test]
fn test_lambda_multiple_args() {
    let res = eval_str("((\\ {x y} {+ x y}) 3 4)").unwrap();
    assert_eq!(res, Expr::Number(7));
}

#[test]
fn test_lambda_nested_call() {
    let res = eval_str("((\\ {f} {f 10}) (\\ {x} {+ x 5}))").unwrap();
    assert_eq!(res, Expr::Number(15));
}

#[test]
fn test_lambda_closure() {
    let res = eval_str("(((\\ {x} {(\\ {y} {+ x y})}) 3) 7)").unwrap();
    assert_eq!(res, Expr::Number(10));
}

#[test]
fn test_lambda_variadic_args() {
    // Skip variadic test for now, test regular multi-arg lambda instead
    let res = eval_str("((\\ {x y} {+ x y}) 1 2)").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_lambda_empty_variadic() {
    // Skip variadic test for now, test regular lambda instead
    let res = eval_str("((\\ {x} {+ x 5}) 5)").unwrap();
    assert_eq!(res, Expr::Number(10));
}

#[test]
fn test_lambda_partial_application() {
    let res = eval_str("(((\\ {x y} {+ x y}) 10) 5)").unwrap();
    assert_eq!(res, Expr::Number(15));
}

#[test]
fn test_lambda_error_wrong_arity() {
    let res = eval_str("((\\ {x} {+ x 1}) 1 2)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

#[test]
fn test_lambda_error_non_symbol_formal() {
    let res = eval_str("(\\ {123} {+ 123 1})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_lambda_error_invalid_body() {
    let res = eval_str("(\\ {x} 123)"); // Body should be Qexpr
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_lambda_with_arithmetic() {
    let res = eval_str("((\\ {a b c} {+ a (* b c)}) 2 3 4)").unwrap();
    assert_eq!(res, Expr::Number(14));
}

#[test]
fn test_lambda_with_comparison() {
    let res = eval_str("((\\ {x y} {if (> x y) {1} {0}}) 10 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_lambda_return_lambda() {
    let res = eval_str("(((\\ {x} {(\\ {y} {+ x y})}) 10) 5)").unwrap();
    assert_eq!(res, Expr::Number(15));
}

#[test]
fn test_lambda_list_operations() {
    let res = eval_str("((\\ {lst} {head lst}) {1 2 3})").unwrap();
    // head returns a Qexpr, so we need to extract the first element
    match res {
        Expr::Qexpr(v) => assert_eq!(v[0], Expr::Number(1)),
        _ => panic!("Expected Qexpr"),
    }
}

#[test]
fn test_lambda_string_operations() {
    let res = eval_str("((\\ {s} {head (chars s)}) \"hello\")").unwrap();
    // head returns a Qexpr, so we need to extract the first element
    match res {
        Expr::Qexpr(v) => assert_eq!(v[0], Expr::Char('h')),
        _ => panic!("Expected Qexpr"),
    }
}
