#include "jlisp.h"

// helper funcs

#define LASSERT(args, cond, err)                                               \
  if (!(cond)) {                                                               \
    lval_free(args);                                                           \
    return lval_err(err);                                                      \
  }

lval *lval_num(long x) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_NUM;
  v->num = x;
  return v;
}
lval *lval_err(char *err) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_ERR;
  v->sym = malloc(strlen(err) + 1);
  strcpy(v->sym, err);
  return v;
}
lval *lval_sym(char *s) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_SYM;
  v->sym = malloc(strlen(s) + 1);
  strcpy(v->sym, s);
  return v;
}
lval *lval_sexpr() {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_SEXPR;
  v->cell = NULL;
  v->count = 0;
  return v;
}
lval *lval_qexpr() {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_QEXPR;
  v->cell = NULL;
  v->count = 0;
  return v;
}

// cell modifiers

lval *lval_pop(lval *v, unsigned i) {
  lval *x = (lval *)v->cell[i];
  memmove(&v->cell[i], &v->cell[i + 1], sizeof(lval *) * (v->count - i - 1));

  v->count--;
  v->cell = realloc(v->cell, sizeof(lval *) * v->count);
  return x;
}

lval *lval_take(lval *v, unsigned i) {
  lval *x = lval_pop(v, i);
  lval_free(v);
  return x;
}

lval *lval_join(lval *x, lval *y) {
  while (y->count) {
    x = lval_append(x, lval_pop(y, 0));
  }

  lval_free(y);
  return x;
}

void lval_free(lval *v) {
  switch (v->type) {
  case LVAL_NUM:
    break;
  case LVAL_ERR:
  case LVAL_SYM:
    free(v->sym);
    break;
  case LVAL_QEXPR:
  case LVAL_SEXPR:
    for (unsigned i = 0; i < v->count; i++)
      lval_free((lval *)v->cell[i]);
    free(v->cell);
    break;
  }

  free(v);
}

void lval_expr_print(lval *v, char o, char c) {
  putchar(o);

  for (unsigned i = 0; i < v->count; i++) {
    lval_print((lval *)v->cell[i]);

    if (i != (v->count - 1))
      putchar(' ');
  }

  putchar(c);
}

void lval_print(lval *v) {
  switch (v->type) {
  case LVAL_NUM:
    printf("%li", v->num);
    break;
  case LVAL_ERR:
    printf("error: %s", v->sym);
    break;
  case LVAL_SYM:
    printf("%s", v->sym);
    break;
  case LVAL_SEXPR:
    lval_expr_print(v, '(', ')');
    break;
  case LVAL_QEXPR:
    lval_expr_print(v, '{', '}');
    break;
  }
}

// builtins & eval

lval *builtin_head(lval *v) {
  LASSERT(v, v->count == 1, "Function 'head' passed too many args!");
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR, "Function 'head' incorrect type!");
  LASSERT(v, v->cell[0]->count > 0, "Function 'head' passed {}!");

  lval *x = lval_take(v, 0); // take first elem
  while (x->count > 1)
    lval_free(lval_pop(x, 1)); // delete all but first value
  return x;
}

lval *builtin_tail(lval *v) {
  LASSERT(v, v->count == 1, "Function 'tail' passed too many args!");
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR, "Function 'tail' incorrect type!");
  LASSERT(v, v->cell[0]->count > 0, "Function 'tail' passed {}!");

  lval *x = lval_take(v, 0);
  lval_free(lval_pop(x, 0)); // delete first value
  return x;
}

lval *builtin_list(lval *v) {
  v->type = LVAL_QEXPR;
  return v;
}

lval *builtin_eval(lval *v) {
  LASSERT(v, v->count == 1, "Function 'eval' passed too many args!");
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR, "Function 'eval' incorrect type!");

  lval *x = lval_take(v, 0);
  x->type = LVAL_SEXPR;
  return lval_eval(x);
}
lval *builtin_join(lval *v) {
  for (unsigned i = 0; i < v->count; i++) {
    LASSERT(v, v->cell[i]->type == LVAL_QEXPR,
            "Function 'join' passed incorrect type");
  }

  lval *x = lval_pop(v, 0);

  while (v->count) {
    x = lval_join(x, lval_pop(v, 0)); // transfer all into x
  }
  lval_free(v);
  return x;
}

lval *builtin(lval *v, char *func) {
  if (strcmp("list", func) == 0)
    return builtin_list(v);
  if (strcmp("head", func) == 0)
    return builtin_head(v);
  if (strcmp("tail", func) == 0)
    return builtin_tail(v);
  if (strcmp("join", func) == 0)
    return builtin_join(v);
  if (strcmp("eval", func) == 0)
    return builtin_eval(v);
  if (strstr("+-/*", func))
    return builtin_op(v, func);
  lval_free(v);
  return lval_err("Unknown function");
}

lval *builtin_op(lval *v, char *op) {
  // all children must be num/primite
  for (unsigned i = 0; i < v->count; i++) {
    if (((lval *)v->cell[i])->type != LVAL_NUM) {
      lval_free(v);
      return lval_err("trying to eval an error");
    }
  }

  lval *f = lval_pop(v, 0);

  // singular negation
  if ((strcmp(op, "-") == 0) && (v->count == 0)) {
    f->num = -f->num;
  }

  // fold over arguments
  while (v->count > 0) {
    lval *y = lval_pop(v, 0);

    if (strcmp(op, "+") == 0)
      f->num += y->num;
    if (strcmp(op, "-") == 0)
      f->num -= y->num;
    if (strcmp(op, "*") == 0)
      f->num *= y->num;
    if (strcmp(op, "/") == 0) {
      if (y->num == 0) {
        lval_free(f);
        lval_free(y);
        f = lval_err("tried to divide by zero");
        break;
      }
      f->num /= y->num;
    }

    lval_free(y);
  }

  lval_free(v);
  return f;
}

lval *lval_eval(lval *v) {
  if (v->type == LVAL_SEXPR)
    return lval_eval_sexpr(v);

  return v;
}

lval *lval_eval_sexpr(lval *v) {
  // collapse children
  for (unsigned i = 0; i < v->count; i++)
    v->cell[i] = (struct lval *)lval_eval((lval *)v->cell[i]);

  // propagate error
  for (unsigned i = 0; i < v->count; i++) {
    if (((lval *)v->cell[i])->type == LVAL_ERR) {
      return lval_take(v, i);
    }
  }

  // empty/unit
  if (v->count == 0)
    return v;
  // single expr
  if (v->count == 1)
    return lval_take(v, 0);

  lval *f = lval_pop(v, 0);
  if (f->type != LVAL_SYM) {
    lval_free(f);
    lval_free(v);

    return lval_err("no operation/symbol in list");
  }

  lval *result = builtin(v, f->sym);
  lval_free(f);
  return result;
}

// parsing and main()

lval *lval_read_num(mpc_ast_t *t) {
  errno = 0;
  long n = strtol(t->contents, NULL, 10);
  return errno != ERANGE ? lval_num(n) : lval_err("failed to parse integer");
}

lval *lval_append(lval *p, lval *c) {
  p->count++;
  p->cell = realloc(p->cell, sizeof(lval *) * p->count);
  p->cell[p->count - 1] = (struct lval *)c;
  return p;
}

lval *lval_read(mpc_ast_t *t) {
  // since both number & symbol are terminal, we can just check for substring
  if (strstr(t->tag, "number"))
    return lval_read_num(t);
  if (strstr(t->tag, "symbol"))
    return lval_sym(t->contents);

  // root or sexpr
  lval *v = NULL;
  // need to check complete str
  if (strcmp(t->tag, ">") == 0) {
    v = lval_sexpr();
  } else if (strstr(t->tag, "sexpr")) {
    v = lval_sexpr();
  } else if (strstr(t->tag, "qexpr")) {
    // qexpr
    v = lval_qexpr();
  }

  for (unsigned i = 0; i < t->children_num; i++) {
    if (strcmp(t->children[i]->contents, "(") == 0)
      continue;
    if (strcmp(t->children[i]->contents, ")") == 0)
      continue;
    if (strcmp(t->children[i]->contents, "{") == 0)
      continue;
    if (strcmp(t->children[i]->contents, "}") == 0)
      continue;
    if (strcmp(t->children[i]->tag, "regex") == 0)
      continue;
    v = lval_append(v, lval_read(t->children[i]));
  }
  return v;
}

int main(int argc, char **argv) {
  mpc_parser_t *Number = mpc_new("number");
  mpc_parser_t *Symbol = mpc_new("symbol");
  mpc_parser_t *Sexpr = mpc_new("sexpr");
  mpc_parser_t *Qexpr = mpc_new("qexpr");
  mpc_parser_t *Expr = mpc_new("expr");
  mpc_parser_t *Jlisp = mpc_new("jlisp");

  mpca_lang(MPCA_LANG_DEFAULT, "                        \
      number : /-?[0-9]+/ ;                             \
      symbol : \"list\" | \"head\" | \"tail\" |         \
      \"join\" | \"eval\" |  '+' | '-' | '*' | '/';     \
      sexpr   : '(' <expr>* ')' ;                       \
      qexpr   : '{' <expr>* '}' ;                       \
      expr   : <number> | <symbol> | <sexpr> | <qexpr> ;\
      jlisp  : /^/ <expr>* /$/ ;                        \
    ",
            Number, Symbol, Sexpr, Qexpr, Expr, Jlisp);

  puts("jlisp version 0.0.1");
  puts("Press Ctrl+c to exit!\n");

  while (1) {
    char *input = readline("jlisp> ");
    add_history(input);

    // if (strlen(input) == 0) {
    // free(input);
    // continue;
    // }

    mpc_result_t r;
    if (mpc_parse("<stdin>", input, Jlisp, &r)) {
      lval *x = lval_read(r.output);
      x = lval_eval(x);
      lval_print(x);
      putchar('\n');
      lval_free(x);

      mpc_ast_delete(r.output);
    } else {
      mpc_err_print(r.output);
      mpc_err_delete(r.error);
    }

    free(input);
  }

  mpc_cleanup(6, Number, Symbol, Sexpr, Qexpr, Expr, Jlisp);
  return 0;
}
