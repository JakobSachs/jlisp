# jlisp Language Issues

This document describes limitations and inconsistencies discovered in the jlisp implementation while building a CSV parser.

## 1. Variable Names Cannot Contain Digits

**Location**: `src/grammar.lalrpop:70-72`

The symbol pattern explicitly excludes digits: `[a-zA-Z\-\+_/\\!&<=>\*\^\|\%]+`

**Impact**:
- ❌ Invalid: `x1`, `row2`, `data-123`, `split1`
- ✓ Valid: `x-one`, `row-two`, `data-abc`, `split-one`

**Example**:
```lisp
(def [row1] "test")  ; ERROR: type error in 'def' expected Symbol, got Number
(def [row-one] "test")  ; OK
```

This is more restrictive than most Lisp dialects where alphanumeric identifiers are standard.

!HAS BEEN FIXED WITH LAST COMMIT!

---

## 2. Asymmetric List Extraction Behavior

**Location**: `src/builtin/collections.rs:4-14`

The `head` and `last` functions behave inconsistently:

- `head` wraps its result in a list: `Expr::List(vec![element])`
- `last` returns the element directly without wrapping

**Impact**:
```lisp
(head ["a" "b"])  ; Returns: ["a"]
(last ["a" "b"])  ; Returns: "b"
```

To extract the first element as an unwrapped value:
```lisp
(last (head ["a" "b"]))  ; Returns: "a"
```

**Why This Matters**: When processing lists of strings (like CSV lines), you need `(last (head lines))` to extract the first string for operations like `split`, which expects a string input, not a list containing a string.

**!HAS BEEN FIXED!** Both `head` and `last` now return unwrapped elements consistently. The CSV parser has been updated to use the simpler `(head lines)` syntax.

---

## 3. The `if` Builtin Cannot Return Empty Lists

**Location**: `src/builtin/core.rs:137-148`

The `if` implementation converts both branches to S-expressions using `.into_list()` then wraps them in `Expr::Sexpr()`. This causes empty lists `[]` to become empty S-expressions `()`.

**Impact**:
```lisp
(if 1 [] [999])  ; Returns: (), not []
(def [x] (if 1 [] [999]))
(print x)  ; Prints: ()
```

**Workaround**: Use accumulator-based recursion instead of direct recursion with an empty list base case:

```lisp
; This doesn't work - returns () from base case
(fun [bad-parse lines]
  [if (== (len lines) 0)
    []  ; Becomes () when returned from if
    (join (list 1) (bad-parse (tail lines)))])

; This works - accumulator starts as [] before entering if
(fun [good-parse lines acc]
  [if (== (len lines) 0)
    [acc]  ; Returns acc, which is a list
    [(good-parse (tail lines) (join acc (list 1)))]])
```

---

## Summary

These issues were discovered while implementing a CSV parser that reads a file and returns a 2D array structure. The workarounds required:

1. Using hyphenated variable names instead of numbered ones
2. Using `(last (head list))` to unwrap the first element of a list
3. Using accumulator-based tail recursion to avoid needing empty list returns from `if`

The final working CSV parser can be found in `csv.jl`.
