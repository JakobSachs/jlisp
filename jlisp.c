#include <stdio.h>
#include <stdlib.h>

#include <editline/history.h>
#include <editline/readline.h>

#include "mpc/mpc.h"

typedef enum { LVAL_NUM, LVAL_ERR } lval_type;
typedef enum { LERR_DIV_ZERO, LERR_BAD_OP, LERR_BAD_NUM } lerr;

typedef struct {
  lval_type type;
  union {
    long num;
    lerr err;
  };
} lval;

lval lval_num(long x) { return (lval){LVAL_NUM, {.num = x}}; }
lval lval_err(lerr err) { return (lval){LVAL_ERR, {.err = err}}; }

void lval_print(lval v) {
  char *msg;
  switch (v.type) {
  case LVAL_NUM:
    printf("%li", v.num);
    break;
  case LVAL_ERR:
    switch (v.err) {
    case LERR_DIV_ZERO:
      msg = "Division by zero!";
      break;
    case LERR_BAD_OP:
      msg = "Invalid operator!";
      break;
    case LERR_BAD_NUM:
      msg = "Invalid number!";
      break;
    }
    printf("error: %s", msg);
    break;
  }
}

lval eval_op(lval x, char *op, lval y) {
  if (x.type == LVAL_ERR)
    return x;
  if (y.type == LVAL_ERR)
    return y;

  if (strcmp(op, "+") == 0)
    return lval_num(x.num + y.num);
  if (strcmp(op, "-") == 0)
    return lval_num(x.num - y.num);
  if (strcmp(op, "*") == 0)
    return lval_num(x.num * y.num);
  if (strcmp(op, "/") == 0)
    return y.num == 0 ? lval_err(LERR_DIV_ZERO) : lval_num(x.num / y.num);
  return lval_err(LERR_BAD_OP);
}

lval eval(mpc_ast_t *t) {
  if (strstr(t->tag, "number")) {
    errno = 0;
    long n = strtol(t->contents, NULL, 10);
    return errno != ERANGE ? lval_num(n) : lval_err(LERR_BAD_NUM);
  }

  char *op = t->children[1]->contents;
  lval x = eval(t->children[2]);

  int i = 3;
  while (strstr(t->children[i]->tag, "expr")) {
    x = eval_op(x, op, eval(t->children[i]));
    i++;
  }
  return x;
}

int main(int argc, char **argv) {
  mpc_parser_t *Number = mpc_new("number");
  mpc_parser_t *Operator = mpc_new("operator");
  mpc_parser_t *Expr = mpc_new("expr");
  mpc_parser_t *Jlisp = mpc_new("jlisp");
  mpca_lang(MPCA_LANG_DEFAULT, "\
      number : /-?[0-9]+/ ;\
      operator : '+' | '-' | '*' | '/';\
      expr : <number> | '(' <operator> <expr>+ ')';\
      jlisp : /^/ <operator> <expr>+ /$/ ;\
      ",
            Number, Operator, Expr, Jlisp);

  puts("jlisp version 0.0.1");
  puts("Press Ctrl+c to exit!\n");

  while (1) {
    char *input = readline("jlisp> ");
    add_history(input);

    mpc_result_t r;
    if (mpc_parse("<stdin>", input, Jlisp, &r)) {
      lval result = eval(r.output);
      lval_print(result);
      puts("\n");
      mpc_ast_delete(r.output);
    } else {
      mpc_err_print(r.output);
      mpc_err_delete(r.error);
    }

    free(input);
  }

  mpc_cleanup(4, Number, Operator, Expr, Jlisp);
  return 0;
}
