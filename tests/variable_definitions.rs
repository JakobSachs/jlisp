use jlisp::ast::{Error, Expr};
mod common;
use common::eval_str;

// Test basic def functionality - should define in root environment
#[test]
fn test_def_basic() {
    // Reset environment for clean test
    common::reset_persistent_env();
    // First define the variable
    common::eval_str_persistent("(def {x} 42)").unwrap();
    // Then access it
    let res = common::eval_str_persistent("x").unwrap();
    assert_eq!(res, Expr::Number(42));
}

// Test def with multiple variables
#[test]
fn test_def_multiple() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {a b c} 1 2 3)").unwrap();
    let res = common::eval_str_persistent("(+ a b c)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

// Test def with expression evaluation
#[test]
fn test_def_with_expression() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {x} (+ 10 20))").unwrap();
    let res = common::eval_str_persistent("x").unwrap();
    assert_eq!(res, Expr::Number(30));
}

// Test local assignment (=) should only affect current environment
#[test]
fn test_local_assignment_basic() {
    let res = eval_str("((\\ {x} {(= {y} 100) (+ x y)}) 50)").unwrap();
    assert_eq!(res, Expr::Number(150));
}

// Test local assignment should not affect global scope
#[test]
fn test_local_assignment_isolation() {
    // This should fail because y is defined locally but accessed globally
    let res = eval_str("((\\ {} {(= {y} 10)}) )");
    assert!(res.is_ok());
    // Now try to access y globally - should fail
    let res = eval_str("y");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::UndefinedSymbol { .. }));
}

// Test def inside lambda should create global variables
#[test]
fn test_def_inside_lambda() {
    common::reset_persistent_env();
    // Use a dummy argument to ensure lambda gets called
    common::eval_str_persistent("((\\ {dummy} {(def {global_var} 999)}) 0)").unwrap();
    let res = common::eval_str_persistent("global_var").unwrap();
    assert_eq!(res, Expr::Number(999));
}

// Test variable shadowing - local should shadow global
#[test]
fn test_variable_shadowing() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {x} 100)").unwrap();
    let res = common::eval_str_persistent("((\\ {x} {x}) 200)").unwrap();
    assert_eq!(res, Expr::Number(200));
}

// Test nested environment variable lookup
#[test]
fn test_nested_environment_lookup() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {outer} 10)").unwrap();
    let res =
        common::eval_str_persistent("((\\ {dummy} {(= {inner} 20) (+ outer inner)}) 0)").unwrap();
    assert_eq!(res, Expr::Number(30));
}

// Test that undefined variables raise errors
#[test]
fn test_undefined_variable() {
    let res = eval_str("nonexistent");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::UndefinedSymbol { sym, .. } if sym == "nonexistent"));
}

// Test def with non-symbol should error
#[test]
fn test_def_non_symbol_error() {
    let res = eval_str("(def {123} 5)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

// Test def with wrong arity should error
#[test]
fn test_def_wrong_arity() {
    let res = eval_str("(def {x})"); // Missing value
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

// Test def with mismatched symbol/value count should error
#[test]
fn test_def_mismatched_count() {
    let res = eval_str("(def {x y} 5)"); // 2 symbols, 1 value
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

// Test local assignment with non-symbol should error
#[test]
fn test_local_assignment_non_symbol() {
    let res = eval_str("((\\ {dummy} {(= {456} 10)}) 0)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

// Test variable redefinition with def
#[test]
fn test_variable_redefinition() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {x} 1)").unwrap();
    common::eval_str_persistent("(def {x} 2)").unwrap();
    let res = common::eval_str_persistent("x").unwrap();
    assert_eq!(res, Expr::Number(2));
}

// Test complex nested scoping
#[test]
fn test_complex_scoping() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {a} 1)").unwrap();
    let res = common::eval_str_persistent("((\\ {b} {(= {c} 3) (+ a b c)}) 2)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

// Test lambda closure with variable capture
#[test]
fn test_lambda_closure_variables() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {x} 10)").unwrap();
    let res = common::eval_str_persistent("((\\ {f} {(f 5)}) (\\ {y} {+ x y}))").unwrap();
    assert_eq!(res, Expr::Number(15));
}

// Test that local assignment doesn't leak to parent
#[test]
fn test_local_assignment_no_leak() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {x} 1)").unwrap();
    common::eval_str_persistent("((\\ {} {(= {x} 2)}) )").unwrap();
    let res = common::eval_str_persistent("x").unwrap();
    assert_eq!(res, Expr::Number(1)); // Should still be 1, not 2
}

// Test def with lambda value
#[test]
fn test_def_lambda_value() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {my-func} (\\ {x} {+ x 1}))").unwrap();
    let res = common::eval_str_persistent("(my-func 5)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

// Test multiple statements with variable dependencies
#[test]
fn test_variable_dependencies() {
    common::reset_persistent_env();
    common::eval_str_persistent("(def {a} 5)").unwrap();
    common::eval_str_persistent("(def {b} (* a 2))").unwrap();
    common::eval_str_persistent("(def {c} (+ b 3))").unwrap();
    let res = common::eval_str_persistent("c").unwrap();
    assert_eq!(res, Expr::Number(13)); // a=5, b=10, c=13
}

// Test direct environment inspection - this should expose the core bug
#[test]
fn test_direct_environment_behavior() {
    // This test directly checks if def is working at all
    common::reset_persistent_env();
    let res = common::eval_str_persistent("(def {test_var} 123)");
    assert!(res.is_ok());

    // If def worked, this should succeed. If it fails, def is broken.
    let res = common::eval_str_persistent("test_var");
    if let Err(ref e) = res {
        println!("DEF BUG: Variable not found after def: {:?}", e);
    }
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), Expr::Number(123));
}
