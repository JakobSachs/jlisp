use jlisp::ast::Expr;
mod common;
use common::eval_str;

#[test]
fn test_logic_operations() {
    // Test NOT operation
    let res = eval_str("(not 0)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(not 1)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(not 5)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(not -1)").unwrap();
    assert_eq!(res, Expr::Number(0));

    // Test AND operation
    let res = eval_str("(and 0 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(and 0 1)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(and 1 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(and 1 1)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(and 5 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(and 5 3)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(and 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(and 1 0 3)").unwrap();
    assert_eq!(res, Expr::Number(0));

    // Test OR operation
    let res = eval_str("(or 0 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(or 0 1)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or 1 0)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or 1 1)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or 5 0)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or 0 0 0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(or 0 0 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(1));

    // Test nested operations
    let res = eval_str("(and (or 0 1) (not 0))").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(or (and 1 0) (not 1))").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_bitwise_operations() {
    // Test AND
    let res = eval_str("(& 5 3)").unwrap(); // 101 & 011 = 001
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(& 12 10)").unwrap(); // 1100 & 1010 = 1000
    assert_eq!(res, Expr::Number(8));
    let res = eval_str("(& 15 0)").unwrap(); // 1111 & 0000 = 0000
    assert_eq!(res, Expr::Number(0));

    // Test OR
    let res = eval_str("(| 5 3)").unwrap(); // 101 | 011 = 111
    assert_eq!(res, Expr::Number(7));
    let res = eval_str("(| 12 10)").unwrap(); // 1100 | 1010 = 1110
    assert_eq!(res, Expr::Number(14));
    let res = eval_str("(| 8 0)").unwrap(); // 1000 | 0000 = 1000
    assert_eq!(res, Expr::Number(8));

    // Test XOR
    let res = eval_str("(^ 5 3)").unwrap(); // 101 ^ 011 = 110
    assert_eq!(res, Expr::Number(6));
    let res = eval_str("(^ 12 10)").unwrap(); // 1100 ^ 1010 = 0110
    assert_eq!(res, Expr::Number(6));
    let res = eval_str("(^ 7 7)").unwrap(); // 0111 ^ 0111 = 0000
    assert_eq!(res, Expr::Number(0));

    // Test left shift
    let res = eval_str("(<< 1 3)").unwrap(); // 1 << 3 = 8
    assert_eq!(res, Expr::Number(8));
    let res = eval_str("(<< 5 2)").unwrap(); // 5 << 2 = 20
    assert_eq!(res, Expr::Number(20));
    let res = eval_str("(<< 0 5)").unwrap(); // 0 << 5 = 0
    assert_eq!(res, Expr::Number(0));

    // Test right shift
    let res = eval_str("(>> 8 3)").unwrap(); // 8 >> 3 = 1
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(>> 20 2)").unwrap(); // 20 >> 2 = 5
    assert_eq!(res, Expr::Number(5));
    let res = eval_str("(>> 5 10)").unwrap(); // 5 >> 10 = 0
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_modulo_and_power() {
    // Test modulo
    let res = eval_str("(% 10 3)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(% 15 4)").unwrap();
    assert_eq!(res, Expr::Number(3));
    let res = eval_str("(% 20 5)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(% 7 3)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(% -7 3)").unwrap(); // Rust's modulo behavior
    assert_eq!(res, Expr::Number(-1));

    // Test power
    let res = eval_str("(** 2 3)").unwrap();
    assert_eq!(res, Expr::Number(8));
    let res = eval_str("(** 5 2)").unwrap();
    assert_eq!(res, Expr::Number(25));
    let res = eval_str("(** 3 4)").unwrap();
    assert_eq!(res, Expr::Number(81));
    let res = eval_str("(** 10 0)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(** 1 5)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(** 0 5)").unwrap();
    assert_eq!(res, Expr::Number(0));
}

#[test]
fn test_truncate_function() {
    // Test truncate with floats that are close to integers
    let res = eval_str("(truncate 1.0)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(truncate 0.0)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(truncate -1.0)").unwrap();
    assert_eq!(res, Expr::Number(-1));

    // Test truncate with floats that are very close to integers
    let res = eval_str("(truncate 1.000001)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(truncate 0.999999)").unwrap();
    assert_eq!(res, Expr::Number(0));
    let res = eval_str("(truncate -1.000001)").unwrap();
    assert_eq!(res, Expr::Number(-2)); // since -1.000001 is closer to -2 than -1 

    // Test truncate with floats that are not close to integers
    let res = eval_str("(truncate 1.5)").unwrap();
    assert_eq!(res, Expr::Number(1));
    let res = eval_str("(truncate 3.14)").unwrap();
    assert_eq!(res, Expr::Number(3));
    let res = eval_str("(truncate -2.7)").unwrap();
    assert_eq!(res, Expr::Number(-3));

    // Test truncate with integers (should return as-is)
    let res = eval_str("(truncate 5)").unwrap();
    assert_eq!(res, Expr::Number(5));
    let res = eval_str("(truncate -3)").unwrap();
    assert_eq!(res, Expr::Number(-3));
}

#[test]
fn test_math_functions() {
    // Test sqrt - should always return float now
    let res = eval_str("(sqrt 0)").unwrap();
    assert_eq!(res, Expr::Float(0.0));
    let res = eval_str("(sqrt 1)").unwrap();
    assert_eq!(res, Expr::Float(1.0));
    let res = eval_str("(sqrt 4)").unwrap();
    assert_eq!(res, Expr::Float(2.0));
    let res = eval_str("(sqrt 9)").unwrap();
    assert_eq!(res, Expr::Float(3.0));
    let res = eval_str("(sqrt 16)").unwrap();
    assert_eq!(res, Expr::Float(4.0));

    // Test sqrt with floats
    let res = eval_str("(sqrt 2.0)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 1.4142).abs() < 0.001));
    let res = eval_str("(sqrt 2.5)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 1.5811).abs() < 0.001));

    // Test sqrt with float that's very close to integer - should still return float
    let res = eval_str("(sqrt 4.00000001)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 2.0).abs() < 0.001));
    assert!(matches!(res, Expr::Float(_))); // Ensure it's Float, not Number

    // Test sqrt with float that's very close to integer - should still return float
    let res = eval_str("(sqrt 4.00000001)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 2.0).abs() < 0.001));
    assert!(matches!(res, Expr::Float(_))); // Ensure it's Float, not Number

    // Test abs - should preserve input type
    let res = eval_str("(abs 5)").unwrap();
    assert_eq!(res, Expr::Number(5));
    let res = eval_str("(abs -5)").unwrap();
    assert_eq!(res, Expr::Number(5));
    let res = eval_str("(abs 0)").unwrap();
    assert_eq!(res, Expr::Number(0));

    // Test abs with floats
    let res = eval_str("(abs 3.14)").unwrap();
    assert_eq!(res, Expr::Float(3.14));
    let res = eval_str("(abs -3.14)").unwrap();
    assert_eq!(res, Expr::Float(3.14));

    // Test min with integers - should return integer
    let res = eval_str("(min 5 3)").unwrap();
    assert_eq!(res, Expr::Number(3));
    let res = eval_str("(min -2 7)").unwrap();
    assert_eq!(res, Expr::Number(-2));
    let res = eval_str("(min 0 0)").unwrap();
    assert_eq!(res, Expr::Number(0));

    // Test min with floats - should return float
    let res = eval_str("(min 3.5 2.7)").unwrap();
    assert_eq!(res, Expr::Float(2.7));
    let res = eval_str("(min 5 2.7)").unwrap();
    assert_eq!(res, Expr::Float(2.7));
    let res = eval_str("(min 3.5 2)").unwrap();
    assert_eq!(res, Expr::Float(2.0));

    // Test max with integers - should return integer
    let res = eval_str("(max 5 3)").unwrap();
    assert_eq!(res, Expr::Number(5));
    let res = eval_str("(max -2 7)").unwrap();
    assert_eq!(res, Expr::Number(7));
    let res = eval_str("(max 0 0)").unwrap();
    assert_eq!(res, Expr::Number(0));

    // Test max with floats - should return float
    let res = eval_str("(max 3.5 2.7)").unwrap();
    assert_eq!(res, Expr::Float(3.5));
    let res = eval_str("(max 5 2.7)").unwrap();
    assert_eq!(res, Expr::Float(5.0));
    let res = eval_str("(max 3.5 2)").unwrap();
    assert_eq!(res, Expr::Float(3.5));

    // Test floor - should always return float now
    let res = eval_str("(floor 3.7)").unwrap();
    assert_eq!(res, Expr::Float(3.0));
    let res = eval_str("(floor 3.2)").unwrap();
    assert_eq!(res, Expr::Float(3.0));
    let res = eval_str("(floor -3.7)").unwrap();
    assert_eq!(res, Expr::Float(-4.0));
    let res = eval_str("(floor -3.2)").unwrap();
    assert_eq!(res, Expr::Float(-4.0));

    // Test ceil - should always return float now
    let res = eval_str("(ceil 3.7)").unwrap();
    assert_eq!(res, Expr::Float(4.0));
    let res = eval_str("(ceil 3.2)").unwrap();
    assert_eq!(res, Expr::Float(4.0));
    let res = eval_str("(ceil -3.7)").unwrap();
    assert_eq!(res, Expr::Float(-3.0));
    let res = eval_str("(ceil -3.2)").unwrap();
    assert_eq!(res, Expr::Float(-3.0));

    // Test round - should always return float now
    let res = eval_str("(round 3.7)").unwrap();
    assert_eq!(res, Expr::Float(4.0));
    let res = eval_str("(round 3.2)").unwrap();
    assert_eq!(res, Expr::Float(3.0));
    let res = eval_str("(round -3.7)").unwrap();
    assert_eq!(res, Expr::Float(-4.0));
    let res = eval_str("(round -3.2)").unwrap();
    assert_eq!(res, Expr::Float(-3.0));
    let res = eval_str("(round 3.5)").unwrap();
    assert_eq!(res, Expr::Float(4.0));
    let res = eval_str("(round -3.5)").unwrap();
    assert_eq!(res, Expr::Float(-4.0));
}

#[test]
fn test_trigonometric_functions() {
    // Test sin - should always return float now
    let res = eval_str("(sin 0)").unwrap();
    assert_eq!(res, Expr::Float(0.0));

    // Test sin with floats
    let res = eval_str("(sin 3.14159)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 0.0).abs() < 0.001));

    // Test cos - should always return float now
    let res = eval_str("(cos 0)").unwrap();
    assert_eq!(res, Expr::Float(1.0));

    // Test cos with floats
    let res = eval_str("(cos 3.14159)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - (-1.0)).abs() < 0.001));

    // Test tan - should always return float now
    let res = eval_str("(tan 0)").unwrap();
    assert_eq!(res, Expr::Float(0.0));

    // Test tan with floats
    let res = eval_str("(tan 1.0)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 1.5574).abs() < 0.001));

    // Test log - should always return float now
    let res = eval_str("(log 1)").unwrap();
    assert_eq!(res, Expr::Float(0.0));

    // Test log with floats
    let res = eval_str("(log 2.71828)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 1.0).abs() < 0.001));

    // Test exp - should always return float now
    let res = eval_str("(exp 0)").unwrap();
    assert_eq!(res, Expr::Float(1.0));

    // Test exp with floats
    let res = eval_str("(exp 1.0)").unwrap();
    assert!(matches!(res, Expr::Float(f) if (f - 2.71828).abs() < 0.001));
}
