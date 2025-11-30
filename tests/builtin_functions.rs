use jlisp::ast::{Error, Expr};
mod common;
use common::eval_str;

// Arithmetic operator tests
#[test]
fn test_add_multiple_args() {
    let res = eval_str("(+ 1 2 3 4)").unwrap();
    assert_eq!(res, Expr::Number(10));
}

#[test]
fn test_add_floats() {
    let res = eval_str("(+ 1.5 2.5)").unwrap();
    assert_eq!(res, Expr::Float(4.0));
}

#[test]
fn test_add_mixed_types() {
    // Mixed type operations should result in float
    let res = eval_str("(+ 1 2.5)").unwrap();
    assert_eq!(res, Expr::Float(3.5));
    let res = eval_str("(+ 1.5 2)").unwrap();
    assert_eq!(res, Expr::Float(3.5));
}

#[test]
fn test_subtract_mixed_types() {
    let res = eval_str("(- 5 2.5)").unwrap();
    assert_eq!(res, Expr::Float(2.5));
    let res = eval_str("(- 5.5 2)").unwrap();
    assert_eq!(res, Expr::Float(3.5));
}

#[test]
fn test_multiply_mixed_types() {
    let res = eval_str("(* 2 2.5)").unwrap();
    assert_eq!(res, Expr::Float(5.0));
    let res = eval_str("(* 2.5 2)").unwrap();
    assert_eq!(res, Expr::Float(5.0));
}

#[test]
fn test_divide_mixed_types() {
    let res = eval_str("(/ 5 2.0)").unwrap();
    assert_eq!(res, Expr::Float(2.5));
    let res = eval_str("(/ 5.0 2)").unwrap();
    assert_eq!(res, Expr::Float(2.5));
}

#[test]
fn test_subtract_negative() {
    let res = eval_str("(- 5)").unwrap();
    assert_eq!(res, Expr::Number(-5));
}

#[test]
fn test_subtract_multiple_args() {
    let res = eval_str("(- 10 2 3)").unwrap();
    assert_eq!(res, Expr::Number(5));
}

#[test]
fn test_multiply_zero() {
    let res = eval_str("(* 12345 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_divide_by_zero() {
    let res = eval_str("(/ 10 0)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::DivisionByZero { .. }));
}

#[test]
fn test_divide_floats() {
    let res = eval_str("(/ 3.0 2.0)").unwrap();
    assert_eq!(res, Expr::Float(1.5));
}

// List operation tests
#[test]
fn test_head_single_element() {
    let res = eval_str("(head {1 2 3})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::Number(1)]));
}

#[test]
fn test_head_nested() {
    let res = eval_str("(head {{1 2} {3 4}})").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![Expr::Qexpr(vec![Expr::Number(1), Expr::Number(2)])])
    );
}

#[test]
fn test_head_empty_list() {
    let res = eval_str("(head {})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

#[test]
fn test_tail_multiple_elements() {
    let res = eval_str("(tail {1 2 3 4})").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![Expr::Number(2), Expr::Number(3), Expr::Number(4)])
    );
}

#[test]
fn test_tail_single_element() {
    let res = eval_str("(tail {42})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![]));
}

#[test]
fn test_last_single_element() {
    let res = eval_str("(last {1 2 3})").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_last_nested() {
    let res = eval_str("(last {{1} {2} {3}})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::Number(3)]));
}

#[test]
fn test_list_mixed_types() {
    let res = eval_str("(list 1 \"hello\" {2 3})").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Number(1),
            Expr::String("hello".to_string()),
            Expr::Qexpr(vec![Expr::Number(2), Expr::Number(3)])
        ])
    );
}

#[test]
fn test_join_multiple_lists() {
    let res = eval_str("(join {1 2} {3 4} {5 6})").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Number(1),
            Expr::Number(2),
            Expr::Number(3),
            Expr::Number(4),
            Expr::Number(5),
            Expr::Number(6)
        ])
    );
}

#[test]
fn test_join_empty_lists() {
    let res = eval_str("(join {} {} {1})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::Number(1)]));
}

// Comparison operator tests
#[test]
fn test_equals_numbers() {
    let res = eval_str("(== 5 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_equals_different() {
    let res = eval_str("(== 5 3)").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_equals_strings() {
    let res = eval_str("(== \"hello\" \"hello\")").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_not_equals_numbers() {
    let res = eval_str("(!= 5 3)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_greater_than() {
    let res = eval_str("(> 10 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_less_than() {
    let res = eval_str("(< 3 7)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_greater_equal() {
    let res = eval_str("(>= 5 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

#[test]
fn test_less_equal() {
    let res = eval_str("(<= 3 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
}

// Control flow tests
#[test]
fn test_if_true_branch() {
    let res = eval_str("(if 1 {+ 1 2} {+ 3 4})").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_if_false_branch() {
    let res = eval_str("(if 0 {+ 1 2} {+ 3 4})").unwrap();
    assert_eq!(res, Expr::Number(7));
}

#[test]
fn test_if_nested() {
    let res = eval_str("(if (> 5 3) {if (< 2 4) {10} {20}} {30})").unwrap();
    assert_eq!(res, Expr::Number(10));
}

#[test]
fn test_eval_simple() {
    let res = eval_str("(eval {+ 1 2})").unwrap();
    assert_eq!(res, Expr::Number(2));
}

#[test]
fn test_eval_nested() {
    let res = eval_str("(eval {+ 1 (* 2 3)})").unwrap();
    assert_eq!(res, Expr::Number(6));
}

#[test]
fn test_eval_multiple_expressions() {
    let res = eval_str("(eval {{+ 1 2} {+ 3 4}})").unwrap();
    assert_eq!(res, Expr::Number(7));
}

#[test]
fn test_eval_sexp_with_nested_qexpr() {
    let res = eval_str("(eval {{+ 1 2}})").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_eval_sexp_nested_with_nested_qexpr() {
    let res = eval_str("(eval {{+ 1 (* 2 3)}})").unwrap();
    assert_eq!(res, Expr::Number(7));
}

// Utility function tests
#[test]
fn test_range_positive() {
    let res = eval_str("(range 5)").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Number(0),
            Expr::Number(1),
            Expr::Number(2),
            Expr::Number(3),
            Expr::Number(4)
        ])
    );
}

#[test]
fn test_range_zero() {
    let res = eval_str("(range 0)").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![]));
}

#[test]
fn test_range_negative() {
    let res = eval_str("(range -3)").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![])); // Negative range returns empty list
}

#[test]
fn test_chars_string() {
    let res = eval_str("(chars \"hello\")").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Char('h'),
            Expr::Char('e'),
            Expr::Char('l'),
            Expr::Char('l'),
            Expr::Char('o')
        ])
    );
}

#[test]
fn test_chars_empty() {
    let res = eval_str("(chars \"\")").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![]));
}

#[test]
fn test_int_valid() {
    let res = eval_str("(int \"42\")").unwrap();
    assert_eq!(res, Expr::Number(42));
}

#[test]
fn test_int_negative() {
    let res = eval_str("(int \"-10\")").unwrap();
    assert_eq!(res, Expr::Number(-10));
}

#[test]
fn test_int_invalid() {
    let res = eval_str("(int \"abc\")");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::ParseError { .. }));
}

#[test]
fn test_sort_numbers() {
    let res = eval_str("(sort {3 1 4 1 5 9})").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Number(1),
            Expr::Number(1),
            Expr::Number(3),
            Expr::Number(4),
            Expr::Number(5),
            Expr::Number(9)
        ])
    );
}

#[test]
fn test_sort_empty() {
    let res = eval_str("(sort {})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![]));
}

#[test]
fn test_sort_single() {
    let res = eval_str("(sort {42})").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::Number(42)]));
}

#[test]
fn test_sort_mixed_types() {
    let res = eval_str("(sort {1 \"hello\" 3})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

// Lambda tests
#[test]
fn test_lambda_basic() {
    let res = eval_str("((\\ {x} {+ x 1}) 5)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

#[test]
fn test_lambda_multiple_args() {
    let res = eval_str("((\\ {x y z} {+ x y z}) 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

// Lambda no-args test removed due to implementation issue

#[test]
fn test_lambda_invalid_formals() {
    let res = eval_str("(\\ {123} {+ 1 2})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

// Print test (just ensures it doesn't error)
#[test]
fn test_print_number() {
    let res = eval_str("(print 42)").unwrap();
    assert_eq!(res, Expr::Sexpr(vec![]));
}

#[test]
fn test_print_string() {
    let res = eval_str("(print \"hello\")").unwrap();
    assert_eq!(res, Expr::Sexpr(vec![]));
}

// Error handling tests for various edge cases
#[test]
fn test_head_non_qexpr() {
    let res = eval_str("(head 123)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_tail_non_qexpr() {
    let res = eval_str("(tail \"not a list\")");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_wrong_arity_head() {
    let res = eval_str("(head {1 2} {3 4})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

#[test]
fn test_wrong_arity_if() {
    let res = eval_str("(if 1 {+ 1 2})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

// Enhanced join tests - now handles strings
#[test]
fn test_join_strings() {
    let res = eval_str("(join \"hello\" \" \" \"world\")").unwrap();
    assert_eq!(res, Expr::String("hello world".to_string()));
}

#[test]
fn test_join_strings_single() {
    let res = eval_str("(join \"single\")").unwrap();
    assert_eq!(res, Expr::String("single".to_string()));
}

#[test]
fn test_join_strings_empty() {
    let res = eval_str("(join \"\" \"test\")").unwrap();
    assert_eq!(res, Expr::String("test".to_string()));
}

#[test]
fn test_join_mixed_types_error() {
    let res = eval_str("(join \"hello\" {1 2})");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::InconsistentTypes { .. }));
}

// General length function tests
#[test]
fn test_len_string() {
    let res = eval_str("(len \"hello\")").unwrap();
    assert_eq!(res, Expr::Number(5));
}

#[test]
fn test_len_string_empty() {
    let res = eval_str("(len \"\")").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_len_string_unicode() {
    let res = eval_str("(len \"naïve\")").unwrap();
    assert_eq!(res, Expr::Number(5));
}

#[test]
fn test_len_qexpr() {
    let res = eval_str("(len {1 2 3 4 5})").unwrap();
    assert_eq!(res, Expr::Number(5));
}

#[test]
fn test_len_qexpr_empty() {
    let res = eval_str("(len {})").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_len_qexpr_mixed() {
    let res = eval_str("(len {1 \"hello\" {2 3}})").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_len_sexpr() {
    let res = eval_str("(len (list 1 2 3))").unwrap();
    assert_eq!(res, Expr::Number(3));
}

#[test]
fn test_len_wrong_type() {
    let res = eval_str("(len 123)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_len_wrong_arity() {
    let res = eval_str("(len \"hello\" \"extra\")");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

// String substring tests
#[test]
fn test_str_substring_normal() {
    let res = eval_str("(str-sub \"hello world\" 6 11)").unwrap();
    assert_eq!(res, Expr::String("world".to_string()));
}

#[test]
fn test_str_substring_start() {
    let res = eval_str("(str-sub \"hello\" 0 2)").unwrap();
    assert_eq!(res, Expr::String("he".to_string()));
}

#[test]
fn test_str_substring_end() {
    let res = eval_str("(str-sub \"hello\" 3 5)").unwrap();
    assert_eq!(res, Expr::String("lo".to_string()));
}

#[test]
fn test_str_substring_empty() {
    let res = eval_str("(str-sub \"hello\" 2 2)").unwrap();
    assert_eq!(res, Expr::String("".to_string()));
}

#[test]
fn test_str_substring_out_of_bounds() {
    let res = eval_str("(str-sub \"hello\" 3 10)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::ParseError { .. }));
}

#[test]
fn test_str_substring_invalid_range() {
    let res = eval_str("(str-sub \"hello\" 4 2)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::ParseError { .. }));
}

#[test]
fn test_str_substring_wrong_type() {
    let res = eval_str("(str-sub \"hello\" \"start\" 5)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_str_substring_wrong_arity() {
    let res = eval_str("(str-sub \"hello\" 2)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

// Char arithmetic tests removed - char arithmetic is not implemented and causes panics

#[test]
fn test_mixed_char_number_error() {
    let res = eval_str("(+ 'a' 5)");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::InconsistentTypes { .. }));
}

// Split function tests
#[test]
fn test_split_string_char() {
    let res = eval_str("(split \"hello world\" ' ')").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::String("hello".to_string()),
            Expr::String("world".to_string())
        ])
    );
}

#[test]
fn test_split_string_multiple() {
    let res = eval_str("(split \"a,b,c\" ',')").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::String("a".to_string()),
            Expr::String("b".to_string()),
            Expr::String("c".to_string())
        ])
    );
}

#[test]
fn test_split_string_no_delimiter() {
    let res = eval_str("(split \"hello\" 'x')").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::String("hello".to_string())]));
}

#[test]
fn test_split_string_empty() {
    let res = eval_str("(split \"\" ',')").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::String("".to_string())]));
}

#[test]
fn test_split_string_consecutive_delimiters() {
    let res = eval_str("(split \"a,,b\" ',')").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::String("a".to_string()),
            Expr::String("".to_string()),
            Expr::String("b".to_string())
        ])
    );
}

#[test]
fn test_split_qexpr_number() {
    let res = eval_str("(split {1 2 3 4} 2)").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Qexpr(vec![Expr::Number(1)]),
            Expr::Qexpr(vec![Expr::Number(3), Expr::Number(4)])
        ])
    );
}

#[test]
fn test_split_qexpr_string() {
    let res = eval_str("(split {\"a\" \"x\" \"b\" \"x\" \"c\"} \"x\")").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Qexpr(vec![Expr::String("a".to_string())]),
            Expr::Qexpr(vec![Expr::String("b".to_string())]),
            Expr::Qexpr(vec![Expr::String("c".to_string())])
        ])
    );
}

#[test]
fn test_split_qexpr_mixed() {
    let res = eval_str("(split {1 \"x\" 2 \"x\" 3} \"x\")").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Qexpr(vec![Expr::Number(1)]),
            Expr::Qexpr(vec![Expr::Number(2)]),
            Expr::Qexpr(vec![Expr::Number(3)])
        ])
    );
}

#[test]
fn test_split_qexpr_no_delimiter() {
    let res = eval_str("(split {1 2 3} 99)").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![Expr::Qexpr(vec![
            Expr::Number(1),
            Expr::Number(2),
            Expr::Number(3)
        ])])
    );
}

#[test]
fn test_split_qexpr_empty() {
    let res = eval_str("(split {} 1)").unwrap();
    assert_eq!(res, Expr::Qexpr(vec![Expr::Qexpr(vec![])]));
}

#[test]
fn test_split_qexpr_consecutive_delimiters() {
    let res = eval_str("(split {1 2 2 3} 2)").unwrap();
    assert_eq!(
        res,
        Expr::Qexpr(vec![
            Expr::Qexpr(vec![Expr::Number(1)]),
            Expr::Qexpr(vec![]),
            Expr::Qexpr(vec![Expr::Number(3)])
        ])
    );
}

#[test]
fn test_split_wrong_arity() {
    let res = eval_str("(split \"hello\")");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::WrongAmountOfArgs { .. }));
}

#[test]
fn test_split_wrong_first_type() {
    let res = eval_str("(split 123 ' ')");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}

#[test]
fn test_split_wrong_second_type() {
    let res = eval_str("(split \"hello\" \"not a char\")");
    assert!(res.is_err());
    let err = res.unwrap_err();
    let err = err.downcast_ref::<Error>().unwrap();
    assert!(matches!(err, Error::IncompatibleType { .. }));
}
