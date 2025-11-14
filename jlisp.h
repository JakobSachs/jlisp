#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <editline/history.h>
#include <editline/readline.h>

#include "mpc/mpc.h"

typedef enum { LVAL_NUM, LVAL_ERR, LVAL_SYM, LVAL_SEXPR, LVAL_QEXPR } lval_type;

typedef struct lval {
  lval_type type;
  union {
    long num;
    char *sym;
    struct {
      unsigned count;
      struct lval **cell;
    };
  };
} lval;

lval *lval_num(long x);
lval *lval_err(char *s);
lval *lval_sym(char *s);
lval *lval_sexpr();
lval *lval_qexpr();

lval *lval_append(lval *p, lval *c);
lval *lval_pop(lval *v, unsigned i);
lval *lval_take(lval *v, unsigned i);
void lval_free(lval *v);

void lval_expr_print(lval *v, char o, char c);

void lval_print(lval *v);

// lval eval(mpc_ast_t *t);
// lval eval_op(lval x, char *op, lval y);
lval *builtin_op(lval *v, char *op);
lval *lval_eval(lval *v);
lval *lval_eval_sexpr(lval *v);

lval *lval_read(mpc_ast_t *t);
lval *lval_read_num(mpc_ast_t *t);
lval *lval_read_symbol(mpc_ast_t *t);
