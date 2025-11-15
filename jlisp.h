#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <editline/history.h>
#include <editline/readline.h>

#include "mpc/mpc.h"

typedef enum {
  LVAL_NUM,
  LVAL_STR,
  LVAL_ERR,
  LVAL_FUN,
  LVAL_SYM,
  LVAL_SEXPR,
  LVAL_QEXPR
} lval_type;

char *ltype_name(lval_type t) {
  switch (t) {
  case LVAL_FUN:
    return "Function";
  case LVAL_NUM:
    return "Number";
  case LVAL_STR:
    return "String";
  case LVAL_ERR:
    return "Error";
  case LVAL_SYM:
    return "Symbol";
  case LVAL_SEXPR:
    return "S-expr";
  case LVAL_QEXPR:
    return "Q-expr";
  default:
    return "Unknown-type";
  }
}

struct lval;
struct lenv;

typedef struct lval lval;
typedef struct lenv lenv;

typedef lval *(*lbuiltin)(lenv *, lval *);

struct lval {
  lval_type type;
  union {
    long num;
    char *sym;
    struct { // expression
      unsigned count;
      lval **cell;
    };
    struct {            // lambda
      lbuiltin builtin; // is NULL if lambda
      lenv *env;
      lval *formals;
      lval *body;
    };
  };
};

struct lenv {
  lenv *par;
  unsigned count;
  char **syms;
  lval **lvals;
};

lenv *lenv_new();
void lenv_free(lenv *e);
lval *lenv_get(lenv *e, lval *k);
void lenv_put(lenv *e, lval *k, lval *v);
void lenv_def(lenv *e, lval *k, lval *v);
lenv *lenv_duplicate(lenv *e);

void lenv_add_builtin(lenv *e, char *name, lbuiltin builtin);

lval *lval_builtin(lbuiltin builtin);
lval *lval_num(long x);
lval *lval_err(char *fmt, ...);
lval *lval_sym(char *s);
lval *lval_str(char *s);
lval *lval_sexpr();
lval *lval_qexpr();

lval *lval_call(lenv *e, lval *f, lval *a);

lval *lval_append(lval *p, lval *c);
lval *lval_pop(lval *v, unsigned i);
lval *lval_take(lval *v, unsigned i);
lval *lval_duplicate(lval *v);
void lval_free(lval *v);

void lval_expr_print(lval *v, char o, char c);

void lval_print(lval *v);

// lval eval(mpc_ast_t *t);
// lval eval_op(lval x, char *op, lval y);
lval *builtin_op(lenv *e, lval *v, char *op);
lval *builtin_lambda(lenv *e, lval *v);
lval *builtin_eval(lenv *e, lval *v);
lval *builtin_list(lenv *e, lval *v);
lval *builtin_load(lenv *e, lval*a);

lval *lval_eval(lenv *e, lval *v);
lval *lval_eval_sexpr(lenv *e, lval *v);

lval *lval_read(mpc_ast_t *t);
lval *lval_read_num(mpc_ast_t *t);
lval *lval_read_symbol(mpc_ast_t *t);

mpc_parser_t *Number;
mpc_parser_t *Symbol;
mpc_parser_t *String;
mpc_parser_t *Sexpr;
mpc_parser_t *Qexpr;
mpc_parser_t *Expr;
mpc_parser_t *Comment;
mpc_parser_t *Jlisp;
