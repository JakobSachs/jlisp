macro_rules! single_list_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {{
        expect_arity($func, &$args, 1, $line)?;
        let ls = $args.into_iter().next().unwrap().into_list($func, $line)?;
        expect_nonempty($func, &ls, $line)?;
        $op(ls)
    }};
}

macro_rules! two_number_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {{
        expect_arity($func, &$args, 2, $line)?;
        let l = $args[0].clone().into_number($func, $line)?;
        let r = $args[1].clone().into_number($func, $line)?;
        Ok(Expr::Number($op(l, r) as i32))
    }};
}

macro_rules! single_string_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {{
        expect_arity($func, &$args, 1, $line)?;
        let s = $args
            .into_iter()
            .next()
            .unwrap()
            .into_string($func, $line)?;
        $op(s, $func, $line)
    }};
}

pub(crate) use single_list_op;
pub(crate) use single_string_op;
pub(crate) use two_number_op;
