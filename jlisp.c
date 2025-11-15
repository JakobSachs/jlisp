#include "jlisp.h"

// helper funcs

#define LASSERT(args, cond, fmt, ...)                                          \
  if (!(cond)) {                                                               \
    lval *err = lval_err(fmt, ##__VA_ARGS__);                                  \
    lval_free(args);                                                           \
    return err;                                                                \
  }

lenv *lenv_new() {
  lenv *e = (lenv *)malloc(sizeof(lenv));
  e->count = 0;
  e->syms = NULL;
  e->lvals = NULL;
  return e;
}
void lenv_free(lenv *e) {
  for (unsigned i = 0; i < e->count; i++) {
    free(e->syms[i]);
    lval_free(e->lvals[i]);
  }

  free(e->syms);
  free(e->lvals);
  free(e);
}

lval *lenv_get(lenv *e, lval *k) {
  for (unsigned i = 0; i < e->count; i++) {
    if (strcmp(e->syms[i], k->sym) == 0)
      return lval_duplicate(e->lvals[i]);
  }

  return lval_err("unbound symbol: %s", k->sym);
}

void lenv_put(lenv *e, lval *k, lval *v) {
  for (unsigned i = 0; i < e->count; i++) {
    if (strcmp(e->syms[i], k->sym) == 0) {
      // overwrite
      lval_free(e->lvals[i]);
      e->lvals[i] = lval_duplicate(v);
      return;
    }
  }

  // not present
  e->count++;
  e->lvals = realloc(e->lvals, sizeof(lval *) * e->count);
  e->syms = realloc(e->syms, sizeof(char *) * e->count);

  e->lvals[e->count - 1] = lval_duplicate(v);
  e->syms[e->count - 1] = malloc(strlen(k->sym) + 1);
  strcpy(e->syms[e->count - 1], k->sym);
}

void lenv_add_builtin(lenv *e, char *name, lbuiltin func) {
  lval *k = lval_sym(name);
  lval *v = lval_fun(func);
  lenv_put(e, k, v);
  lval_free(k);
  lval_free(v);
}

lval *lval_fun(lbuiltin func) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_FUN;
  v->fun = func;
  return v;
}

lval *lval_num(long x) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_NUM;
  v->num = x;
  return v;
}

lval *lval_err(char *fmt, ...) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_ERR;

  va_list va;
  va_start(va, fmt);

  v->sym = malloc(512);

  vsnprintf(v->sym, 511, fmt, va);

  v->sym = realloc(v->sym, strlen(v->sym) + 1);

  va_end(va);

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

lval *lval_duplicate(lval *v) {
  lval *out = malloc(sizeof(lval));
  out->type = v->type;

  switch (v->type) {
  case LVAL_FUN:
    out->fun = v->fun;
    break;

  case LVAL_NUM:
    out->num = v->num;
    break;

  case LVAL_SYM:
  case LVAL_ERR:
    out->sym = malloc(strlen(v->sym) + 1);
    strcpy(out->sym, v->sym);
    break;

  case LVAL_SEXPR:
  case LVAL_QEXPR:
    out->count = v->count;
    out->cell = malloc(sizeof(lval *) * v->count);
    for (unsigned i = 0; i < v->count; i++) {
      out->cell[i] = lval_duplicate(v->cell[i]);
    }
    break;
  };

  return out;
}

void lval_free(lval *v) {
  switch (v->type) {
  case LVAL_NUM:
  case LVAL_FUN:
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
  case LVAL_FUN:
    printf("<function>");
    break;
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

lval *builtin_head(lenv *e, lval *v) {
  LASSERT(v, v->count == 1,
          "Function 'head' passed too many args! Got %u, expected 1", v->count);
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'head' incorrect type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));
  LASSERT(v, v->cell[0]->count > 0, "Function 'head' passed {}!");

  lval *x = lval_take(v, 0); // take first elem
  while (x->count > 1)
    lval_free(lval_pop(x, 1)); // delete all but first value
  return x;
}

lval *builtin_tail(lenv *e, lval *v) {
  LASSERT(v, v->count == 1,
          "Function 'tail' passed too many args! Got %u, expected 1", v->count);
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'tail' incorrect type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));
  LASSERT(v, v->cell[0]->count > 0, "Function 'tail' passed {}!");

  lval *x = lval_take(v, 0);
  lval_free(lval_pop(x, 0)); // delete first value
  return x;
}

lval *builtin_list(lenv *e, lval *v) {
  v->type = LVAL_QEXPR;
  return v;
}

lval *builtin_eval(lenv *e, lval *v) {
  LASSERT(v, v->count == 1,
          "Function 'eval' passed too many args! Got %u, expected 1", v->count);
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'eval' incorrect type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));

  lval *x = lval_take(v, 0);
  x->type = LVAL_SEXPR;
  return lval_eval(e, x);
}
lval *builtin_join(lenv *e, lval *v) {
  for (unsigned i = 0; i < v->count; i++) {
    LASSERT(v, v->cell[i]->type == LVAL_QEXPR,
            "Function 'join' incorrect type! Got %s, expected %s",
            ltype_name(v->cell[i]->type), ltype_name(LVAL_QEXPR));
  }

  lval *x = lval_pop(v, 0);

  while (v->count) {
    x = lval_join(x, lval_pop(v, 0)); // transfer all into x
  }
  lval_free(v);
  return x;
}

lval *builtin_def(lenv *e, lval *v) {
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'def' incorrect type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));

  lval *syms = v->cell[0];

  // first list must all be syms
  for (unsigned i = 0; i < syms->count; i++) {
    LASSERT(v, syms->cell[i]->type == LVAL_SYM,
            "Function 'def' cannot define non-symbol (received a %s)",
            ltype_name(syms->cell[i]->type));
  }

  LASSERT(v, syms->count == v->count - 1,
          "Function 'def' received unmatched amounts of syms and values! (%u "
          "symbols an %u values)",
          syms->count, v->count - 1);

  // assign values

  for (unsigned i = 0; i < syms->count; i++) {
    lenv_put(e, syms->cell[i], v->cell[i + 1]);
  }

  lval_free(v);
  return lval_sexpr();
}

lval *builtin_op(lenv *e, lval *v, char *op) {
  // all children must be num/primite
  for (unsigned i = 0; i < v->count; i++) {
    if ((v->cell[i])->type != LVAL_NUM) {
      lval*e =  lval_err("trying to eval an '%s' error on op '%s'",
                      ltype_name(v->cell[i]->type), op);
      lval_free(v);
      return e;
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
        unsigned tmp = f->num;
        lval_free(f);
        lval_free(y);
        f = lval_err("tried to divide by zero (%u / 0)", tmp);
        break;
      }
      f->num /= y->num;
    }

    lval_free(y);
  }

  lval_free(v);
  return f;
}

lval *builtin_add(lenv *e, lval *a) { return builtin_op(e, a, "+"); }
lval *builtin_sub(lenv *e, lval *a) { return builtin_op(e, a, "-"); }
lval *builtin_mul(lenv *e, lval *a) { return builtin_op(e, a, "*"); }
lval *builtin_div(lenv *e, lval *a) { return builtin_op(e, a, "/"); }

lval *lval_eval(lenv *e, lval *v) {
  if (v->type == LVAL_SYM) {
    lval *x = lenv_get(e, v);
    lval_free(v);
    return x;
  }

  if (v->type == LVAL_SEXPR)
    return lval_eval_sexpr(e, v);

  return v;
}

lval *lval_eval_sexpr(lenv *e, lval *v) {
  // collapse children
  for (unsigned i = 0; i < v->count; i++)
    v->cell[i] = lval_eval(e, v->cell[i]);

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
  if (f->type != LVAL_FUN) {
    lval_free(f);
    lval_free(v);

    return lval_err("received an sexpr without a operator in the list");
  }

  lval *result = f->fun(e, v);
  lval_free(f);
  return result;
}

// parsing and main()

lval *lval_read_num(mpc_ast_t *t) {
  errno = 0;
  long n = strtol(t->contents, NULL, 10);
  return errno != ERANGE ? lval_num(n)
                         : lval_err("failed to parse integer: %s", t->contents);
}

lval *lval_append(lval *p, lval *c) {
  p->count++;
  p->cell = realloc(p->cell, sizeof(lval *) * p->count);
  p->cell[p->count - 1] = c;
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

void lenv_add_builtins(lenv *e) {
  lenv_add_builtin(e, "list", builtin_list);
  lenv_add_builtin(e, "head", builtin_head);
  lenv_add_builtin(e, "tail", builtin_tail);
  lenv_add_builtin(e, "eval", builtin_eval);
  lenv_add_builtin(e, "join", builtin_join);
  lenv_add_builtin(e, "def", builtin_def);

  // arithmetics
  lenv_add_builtin(e, "+", builtin_add);
  lenv_add_builtin(e, "-", builtin_sub);
  lenv_add_builtin(e, "*", builtin_mul);
  lenv_add_builtin(e, "/", builtin_div);
}

int main(int argc, char **argv) {
  lenv *e = lenv_new();
  lenv_add_builtins(e);

  mpc_parser_t *Number = mpc_new("number");
  mpc_parser_t *Symbol = mpc_new("symbol");
  mpc_parser_t *Sexpr = mpc_new("sexpr");
  mpc_parser_t *Qexpr = mpc_new("qexpr");
  mpc_parser_t *Expr = mpc_new("expr");
  mpc_parser_t *Jlisp = mpc_new("jlisp");

  mpca_lang(MPCA_LANG_DEFAULT, "                          \
      number  : /-?[0-9]+/ ;                              \
      symbol  : /[a-zA-Z0-9_+\\-*\\/\\\\=<>!&]+/ ;        \
      sexpr   : '(' <expr>* ')' ;                         \
      qexpr   : '{' <expr>* '}' ;                         \
      expr    : <number> | <symbol> | <sexpr> | <qexpr> ; \
      jlisp   : /^/ <expr>* /$/ ;                         \
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
      x = lval_eval(e, x);
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
  lenv_free(e);
  return 0;
}
