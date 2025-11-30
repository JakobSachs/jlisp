use crate::ast::{Error, Expr};

pub fn builtin_op(sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    // Handle bitwise operations separately (only work on integers)
    if matches!(sym, "&" | "|" | "^" | "<<" | ">>") {
        return builtin_bitwise(sym, args, line);
    }

    // Handle modulo and power operations
    if matches!(sym, "%" | "**") {
        return builtin_extended_arith(sym, args, line);
    }

    //check type of first member is valid
    if !matches!(args[0], Expr::Number(_) | Expr::Float(_) | Expr::Char(_)) {
        return Err(Error::IncompatibleType {
            op: sym.to_owned(),
            expected: "Number,Float,Char".to_string(),
            received: args[0].as_str(),
            line,
        });
    }

    // Check if all arguments are chars (char operations work separately)
    let all_chars = args.iter().all(|x| matches!(x, Expr::Char(_)));

    // If all chars, handle separately
    if all_chars {
        return builtin_char_arith(sym, args, line);
    }

    // Check if any argument is a float - if so, result should be float
    let has_float = args.iter().any(|x| matches!(x, Expr::Float(_)));

    // Check if any argument is a char - chars can't mix with numbers/floats
    let has_char = args.iter().any(|x| matches!(x, Expr::Char(_)));

    // For division, we need special handling to determine if result should be float
    let should_return_float = has_float;

    // Reject mixing chars with numbers/floats
    if has_char && (has_float || args.iter().any(|x| matches!(x, Expr::Number(_)))) {
        return Err(Error::InconsistentTypes {
            op: sym.to_owned(),
            line,
        });
    }

    if sym == "-" && args.len() == 1 {
        match args[0] {
            Expr::Number(n) => {
                if should_return_float {
                    return Ok(Expr::Float(-(n as f32)));
                } else {
                    return Ok(Expr::Number(-n));
                }
            }
            Expr::Float(f) => return Ok(Expr::Float(-f)),
            _ => {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number,Float".to_string(),
                    received: args[0].as_str(),
                    line,
                });
            }
        }
    }

    // Handle division specially to determine if result should be float
    if sym == "/" && !has_float {
        // Pure integer division - check if result is exact
        let func: fn(i32, i32) -> i32 = match sym {
            "/" => |a, b| a / b,
            _ => panic!(),
        };

        let start = match &args[0] {
            Expr::Number(n) => *n,
            _ => panic!(),
        };
        let mut out = start;

        for v in args.iter().skip(1) {
            let Expr::Number(v) = v else {
                panic!();
            };
            // check for valid op/not-zero
            if *v == 0 {
                return Err(Error::DivisionByZero { line });
            }
            out = func(out, *v);
        }

        // Check if the division was exact by multiplying back
        let mut temp = out;
        for v in args.iter().skip(1).rev() {
            let Expr::Number(v) = v else {
                panic!();
            };
            temp *= *v;
        }

        if temp == start {
            Ok(Expr::Number(out))
        } else {
            // Not exact division, return float
            let mut float_result = start as f32;
            for v in args.iter().skip(1) {
                let Expr::Number(v) = v else {
                    panic!();
                };
                float_result /= *v as f32;
            }
            Ok(Expr::Float(float_result))
        }
    } else if should_return_float {
        // Convert all arguments to float for mixed operations
        let func: fn(f32, f32) -> f32 = match sym {
            "+" => |a, b| a + b,
            "-" => |a, b| a - b,
            "*" => |a, b| a * b,
            "/" => |a, b| a / b,
            _ => panic!(),
        };

        let start = match &args[0] {
            Expr::Number(n) => *n as f32,
            Expr::Float(f) => *f,
            _ => panic!(),
        };
        let mut out = start;

        for v in args.iter().skip(1) {
            let v_float = match v {
                Expr::Number(n) => *n as f32,
                Expr::Float(f) => *f,
                _ => panic!(),
            };
            // check for valid op
            if !(sym == "/" && v_float == 0.0) {
                out = func(out, v_float);
            } else {
                return Err(Error::DivisionByZero { line });
            }
        }
        Ok(Expr::Float(out))
    } else {
        // Pure integer operations
        let func: fn(i32, i32) -> i32 = match sym {
            "+" => |a, b| a + b,
            "-" => |a, b| a - b,
            "*" => |a, b| a * b,
            "/" => |a, b| a / b,
            _ => panic!(),
        };

        let start = match &args[0] {
            Expr::Number(n) => *n,
            _ => panic!(),
        };
        let mut out = start;

        for v in args.iter().skip(1) {
            let Expr::Number(v) = v else {
                panic!();
            };
            // check for valid op/not-zero
            if !(sym == "/" && *v == 0) {
                out = func(out, *v);
            } else {
                return Err(Error::DivisionByZero { line });
            }
        }
        Ok(Expr::Number(out))
    }
}

fn builtin_bitwise(sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if args.is_empty() {
        return Err(Error::IncompatibleType {
            op: sym.to_owned(),
            expected: "At least one argument".to_string(),
            received: "None".to_string(),
            line,
        });
    }

    // Check all arguments are numbers (bitwise ops only work on integers)
    for arg in &args {
        if !matches!(arg, Expr::Number(_)) {
            return Err(Error::IncompatibleType {
                op: sym.to_owned(),
                expected: "Number".to_string(),
                received: arg.as_str(),
                line,
            });
        }
    }

    let start = match &args[0] {
        Expr::Number(n) => *n,
        _ => panic!(),
    };

    match sym {
        "&" | "|" | "^" => {
            let func: fn(i32, i32) -> i32 = match sym {
                "&" => |a, b| a & b,
                "|" => |a, b| a | b,
                "^" => |a, b| a ^ b,
                _ => panic!(),
            };
            let mut out = start;
            for v in args.iter().skip(1) {
                let Expr::Number(v) = v else { panic!() };
                out = func(out, *v);
            }
            Ok(Expr::Number(out))
        }
        "<<" | ">>" => {
            if args.len() != 2 {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Exactly 2 arguments".to_string(),
                    received: format!("{}", args.len()),
                    line,
                });
            }

            let Expr::Number(shift_amount) = args[1] else {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number".to_string(),
                    received: args[1].as_str(),
                    line,
                });
            };

            // Check shift amount is within valid range (0-31 for i32)
            if shift_amount < 0 || shift_amount > 31 {
                return Err(Error::ParseError {
                    msg: "Shift amount must be between 0 and 31".to_string(),
                    line,
                });
            }

            let result = match sym {
                "<<" => start << shift_amount,
                ">>" => start >> shift_amount,
                _ => panic!(),
            };
            Ok(Expr::Number(result))
        }
        _ => panic!(),
    }
}

fn builtin_extended_arith(sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if args.len() != 2 {
        return Err(Error::IncompatibleType {
            op: sym.to_owned(),
            expected: "Exactly 2 arguments".to_string(),
            received: format!("{}", args.len()),
            line,
        });
    }

    // Check if any argument is a float
    let has_float = args.iter().any(|x| matches!(x, Expr::Float(_)));

    match sym {
        "%" => {
            // Modulo only works on integers
            if has_float {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number".to_string(),
                    received: "Float".to_string(),
                    line,
                });
            }

            let Expr::Number(a) = args[0] else {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number".to_string(),
                    received: args[0].as_str(),
                    line,
                });
            };

            let Expr::Number(b) = args[1] else {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number".to_string(),
                    received: args[1].as_str(),
                    line,
                });
            };

            if b == 0 {
                return Err(Error::DivisionByZero { line });
            }
            Ok(Expr::Number(a % b))
        }
        "**" => {
            // Power operation - supports mixed types
            let a = match &args[0] {
                Expr::Number(n) => *n as f32,
                Expr::Float(f) => *f,
                _ => {
                    return Err(Error::IncompatibleType {
                        op: sym.to_owned(),
                        expected: "Number or Float".to_string(),
                        received: args[0].as_str(),
                        line,
                    });
                }
            };

            let b = match &args[1] {
                Expr::Number(n) => *n,
                Expr::Float(f) => {
                    // For float exponents, we need to use powf instead of powi
                    let result = a.powf(*f);
                    return Ok(Expr::Float(result));
                }
                _ => {
                    return Err(Error::IncompatibleType {
                        op: sym.to_owned(),
                        expected: "Number or Float".to_string(),
                        received: args[1].as_str(),
                        line,
                    });
                }
            };

            if b < 0 {
                return Err(Error::ParseError {
                    msg: "Power exponent must be non-negative".to_string(),
                    line,
                });
            }

            let result = a.powi(b);
            if has_float {
                Ok(Expr::Float(result))
            } else {
                // Check if result is close to an integer
                if (result - result.round()).abs() < 0.0001 {
                    Ok(Expr::Number(result.round() as i32))
                } else {
                    Ok(Expr::Float(result))
                }
            }
        }
        _ => panic!(),
    }
}

fn builtin_char_arith(sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if sym == "-" && args.len() == 1 {
        match args[0] {
            Expr::Char(c) => return Ok(Expr::Number(-(c as i32))),
            _ => panic!(),
        }
    }

    let func: fn(i32, i32) -> i32 = match sym {
        "+" => |a, b| a + b,
        "-" => |a, b| a - b,
        "*" => |a, b| a * b,
        "/" => |a, b| a / b,
        _ => panic!(),
    };

    let start = match &args[0] {
        Expr::Char(c) => *c as i32,
        _ => panic!(),
    };
    let mut out = start;

    for v in args.iter().skip(1) {
        let Expr::Char(c) = v else {
            panic!();
        };
        // check for valid op/not-zero
        if !(sym == "/" && *c as i32 == 0) {
            out = func(out, *c as i32);
        } else {
            return Err(Error::DivisionByZero { line });
        }
    }

    // If result is in valid char range and we're doing addition/subtraction, return char
    if matches!(sym, "+" | "-") && out >= 0 && out <= 255 {
        Ok(Expr::Char(out as u8 as char))
    } else {
        Ok(Expr::Number(out))
    }
}
