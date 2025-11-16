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
  e->par = NULL;
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
  // THIS IS BADDDDD
  // env should likely be a hash-map, not this cursed O(n) lookup
  for (unsigned i = 0; i < e->count; i++) {
    if (strcmp(e->syms[i], k->sym) == 0)
      return lval_duplicate(e->lvals[i]);
  }

  if (e->par != NULL) {
    return lenv_get(e->par, k);
  } else {
    return lval_err("unbound symbol: %s", k->sym);
  }
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

void lenv_def(lenv *e, lval *k, lval *v) {
  // def global scope
  while (e->par != NULL) {
    e = e->par;
  }

  lenv_put(e, k, v);
}

lenv *lenv_duplicate(lenv *e) {
  lenv *n = malloc(sizeof(lenv));
  n->par = e->par;
  n->count = e->count;
  n->syms = malloc(sizeof(char *) * e->count);
  n->lvals = malloc(sizeof(lval *) * e->count);
  for (unsigned i = 0; i < e->count; i++) {
    n->syms[i] = malloc(strlen(e->syms[i]) + 1);
    strcpy(n->syms[i], e->syms[i]);
    n->lvals[i] = lval_duplicate(e->lvals[i]);
  }

  return n;
}

void lenv_add_builtin(lenv *e, char *name, lbuiltin builtin) {
  lval *k = lval_sym(name);
  lval *v = lval_builtin(builtin);
  lenv_put(e, k, v);
  lval_free(k);
  lval_free(v);
}

lval *lval_builtin(lbuiltin builtin) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_FUN;
  v->builtin = builtin;
  return v;
}

lval *lval_lambda(lval *formals, lval *body) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_FUN;

  v->builtin = NULL;
  v->env = lenv_new();

  v->formals = formals;
  v->body = body;
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

lval *lval_str(char *s) {
  lval *v = malloc(sizeof(lval));
  v->type = LVAL_STR;
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

lval *lval_call(lenv *e, lval *f, lval *a) {
  if (f->builtin != NULL)
    return f->builtin(e, a);

  unsigned given = a->count;
  unsigned total = f->formals->count;

  while (a->count) {
    if (f->formals->count == 0) {
      lval_free(a);
      return lval_err("Function passed too many arguments! Got %u, expected %u",
                      given, total);
    }

    lval *sym = lval_pop(f->formals, 0);

    // var arg special case
    if (strcmp(sym->sym, "&") == 0) {
      //  & needs another symbol
      if (f->formals->count != 1) {
        lval_free(a);
        return lval_err("Function signature invalid! Symbol '&' not followed "
                        "by single symbol");
      }

      // next forma should be bound to ALL remaining args
      lval *nsym = lval_pop(f->formals, 0);
      lenv_put(f->env, nsym, builtin_list(e, a));
      lval_free(sym);
      lval_free(nsym);
      break; // end loop
    }

    lval *val = lval_pop(a, 0);

    lenv_put(f->env, sym, val); // push to func scope
    lval_free(sym);
    lval_free(val);
  }

  lval_free(a);

  // bind to empty list
  if (f->formals->count > 0 && (strcmp(f->formals->cell[0]->sym, "&") == 0)) {
    if (f->formals->count != 2) {
      return lval_err("Function signature invalid! Sybmol '&' not followed by "
                      "single symbol");
    }

    lval_free(lval_pop(f->formals, 0));

    lval *sym = lval_pop(f->formals, 0);
    lval *val = lval_qexpr();

    lenv_put(f->env, sym, val);
    lval_free(sym);
    lval_free(val);
  }

  // all formals have been bound -> eval
  if (f->formals->count == 0) {
    f->env->par = e;
    return builtin_eval(f->env,
                        lval_append(lval_sexpr(), lval_duplicate(f->body)));
  } else {
    return lval_duplicate(f);
  }
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
    out->builtin = v->builtin;
    if (v->builtin == NULL) { // lambda
      out->env = lenv_duplicate(v->env);
      out->formals = lval_duplicate(v->formals);
      out->body = lval_duplicate(v->body);
    }

    break;

  case LVAL_NUM:
    out->num = v->num;
    break;

  case LVAL_SYM:
  case LVAL_STR:
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
    break;
  case LVAL_FUN:
    if (v->builtin == NULL) {
      lenv_free(v->env);
      lval_free(v->formals);
      lval_free(v->body);
    }
    break;
  case LVAL_ERR:
  case LVAL_STR:
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

void lval_print_str(lval *v) {
  char *escaped = malloc(strlen(v->sym) + 1);
  strcpy(escaped, v->sym);
  escaped = mpcf_escape(escaped);
  printf("\"%s\"", escaped);
  free(escaped);
}

void lval_print(lval *v) {
  switch (v->type) {
  case LVAL_FUN:
    if (v->builtin != NULL) {
      printf("<builtin>");
    } else {
      printf("(\\ ");
      lval_print(v->formals);
      putchar(' ');
      lval_print(v->body);
      putchar(')');
    }
    break;
  case LVAL_NUM:
    printf("%li", v->num);
    break;
  case LVAL_ERR:
    printf("error: %s", v->sym);
    break;
  case LVAL_STR:
    lval_print_str(v);
    break;
  case LVAL_SYM:
    printf("'%s'", v->sym);
    break;
  case LVAL_SEXPR:
    lval_expr_print(v, '(', ')');
    break;
  case LVAL_QEXPR:
    lval_expr_print(v, '{', '}');
    break;
  }
}

unsigned lval_eq(lval *x, lval *y) {
  if (x->type != y->type)
    return 0;

  switch (x->type) {
  case LVAL_NUM:
    return (x->num == y->num);

  // all use sym
  case LVAL_ERR:
  case LVAL_STR:
  case LVAL_SYM:
    return strcmp(x->sym, y->sym) == 0;

  case LVAL_FUN:
    if ((x->builtin != NULL) || (y->builtin != NULL)) {
      return x->builtin == y->builtin;
    } else {
      return lval_eq(x->formals, y->formals) && lval_eq(x->body, y->body);
    }

  case LVAL_QEXPR:
  case LVAL_SEXPR:
    if (x->count != y->count)
      return 0;

    for (unsigned i = 0; i < x->count; i++) {
      if (lval_eq(x->cell[i], y->cell[i]) == 0)
        return 0;
    }

    return 1;
    break;
  }
  // unknown
  return 0;
}

// builtins & eval

lval *builtin_head(__attribute__((unused)) lenv *e, lval *v) {
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
lval *builtin_last(__attribute__((unused)) lenv *e, lval *v) {
  LASSERT(v, v->count == 1,
          "Function 'last' passed too many args! Got %u, expected 1", v->count);
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'last' incorrect type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));
  LASSERT(v, v->cell[0]->count > 0, "Function 'last' passed {}!");

  lval *x = lval_take(v, 0); // take first elem
  while (x->count > 1)
    lval_free(lval_pop(x, 0)); 
  return x;
}

lval *builtin_tail(__attribute__((unused)) lenv *e, lval *v) {
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

lval *builtin_list(__attribute__((unused)) lenv *e, lval *v) {
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
lval *builtin_join(__attribute__((unused)) lenv *e, lval *v) {
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

lval *builtin_var(lenv *e, lval *v, char *func) {
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "function '%s' tried to define with type %s instead of q-expr ", func,
          ltype_name(v->cell[0]->type));

  lval *syms = v->cell[0];
  for (unsigned i = 0; i < syms->count; i++) {
    LASSERT(v, syms->cell[i]->type == LVAL_SYM,
            "Function '%s' cannot define non-symbol. Got %s but expected a "
            "Symbol",
            func, ltype_name(syms->cell[i]->type));
  }

  LASSERT(v, syms->count == (v->count - 1),
          "Function '%s' passed differing amounts of arguments for symbols. "
          "Got %u, expected %u",
          func, syms->count, v->count - 1);

  for (unsigned i = 0; i < syms->count; i++) {
    if (strcmp(func, "def") == 0) {
      lenv_def(e, syms->cell[i], v->cell[i + 1]);
    } else if (strcmp(func, "=") == 0) {
      lenv_put(e, syms->cell[i], v->cell[i + 1]);
    } else {
      lval *err = lval_err("trying to define with an invalid func: %s", func);
      lval_free(v);
      return err;
    }
  }

  lval_free(v);
  return lval_sexpr();
}

lval *builtin_def(lenv *e, lval *v) { return builtin_var(e, v, "def"); }
lval *builtin_put(lenv *e, lval *v) { return builtin_var(e, v, "="); }

lval *builtin_if(lenv *e, lval *v) {
  LASSERT(v, v->count == 3,
          "'if' got wrong amount of arguments. Got %u expected 3", v->count);
  LASSERT(v, v->cell[0]->type == LVAL_NUM,
          "'if' got wrong type for first argument: %s",
          ltype_name(v->cell[0]->type));
  LASSERT(v, v->cell[1]->type == LVAL_QEXPR,
          "'if' got wrong type for second argument: %s",
          ltype_name(v->cell[1]->type));
  LASSERT(v, v->cell[2]->type == LVAL_QEXPR,
          "'if' got wrong type for third argument: %s",
          ltype_name(v->cell[2]->type));

  lval *o;
  v->cell[1]->type = LVAL_SEXPR;
  v->cell[2]->type = LVAL_SEXPR;

  if (v->cell[0]->num) {
    // eval if-branch
    o = lval_eval(e, lval_pop(v, 1));
  } else {
    // else
    o = lval_eval(e, lval_pop(v, 2));
  }
  lval_free(v);
  return o;
}

lval *builtin_print(__attribute__((unused)) lenv *e, lval *a) {
  for (unsigned i = 0; i < a->count; i++) {
    lval_print(a->cell[i]);
    putchar(' ');
  }
  putchar('\n');
  lval_free(a);
  fflush(stdout);
  return lval_sexpr();
}
lval *builtin_error(__attribute__((unused)) lenv *e, lval *a) {
  LASSERT(a, a->count == 1, "error must have only 1 elem");
  LASSERT(a, a->cell[0]->type == LVAL_STR, "error can only print strings");

  lval *err = lval_err(a->cell[0]->sym);
  lval_free(a);
  return err;
}

lval *builtin_op(__attribute__((unused)) lenv *e, lval *v, char *op) {
  // all children must be num/primite
  for (unsigned i = 0; i < v->count; i++) {
    if ((v->cell[i])->type != LVAL_NUM) {
      lval *e = lval_err("trying to eval an '%s' error on op '%s'",
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

lval *builtin_cmp(__attribute__((unused)) lenv *e, lval *a, char *op) {

  LASSERT(a, a->count == 2, "Comparison got passed %u arguments, expected 2",
          a->count);
  int o;

  if (strcmp(op, "==") == 0) {
    o = lval_eq(a->cell[0], a->cell[1]);
  }
  if (strcmp(op, "!=") == 0) {
    o = !lval_eq(a->cell[0], a->cell[1]);
  }

  lval_free(a);
  return lval_num(o);
}

lval *builtin_eq(lenv *e, lval *a) { return builtin_cmp(e, a, "=="); }

lval *builtin_ne(lenv *e, lval *a) { return builtin_cmp(e, a, "!="); }

lval *builtin_ord(__attribute__((unused)) lenv *e, lval *a, char *op) {
  LASSERT(a, a->count == 2, "Ordering got passed %u arguments, expected 2",
          a->count);
  LASSERT(a, a->cell[0]->type == LVAL_NUM,
          "Ordering got passed a %s when it expected a number",
          ltype_name(a->cell[0]->type));
  LASSERT(a, a->cell[1]->type == LVAL_NUM,
          "Ordering got passed a %s when it expected a number",
          ltype_name(a->cell[1]->type));

  unsigned o = 0;

  unsigned l = a->cell[0]->num;
  unsigned r = a->cell[1]->num;
  if (strcmp(op, ">") == 0)
    o = l > r;
  if (strcmp(op, "<") == 0)
    o = l < r;
  if (strcmp(op, ">=") == 0)
    o = l >= r;
  if (strcmp(op, "<=") == 0)
    o = l <= r;

  lval_free(a);
  return lval_num(o);
}

lval *builtin_lambda(__attribute__((unused)) lenv *e, lval *v) {
  LASSERT(v, v->count == 2,
          "Function 'lambda' passed wrong amount of args! Got %u, expected 2",
          v->count);
  LASSERT(v, v->cell[0]->type == LVAL_QEXPR,
          "Function 'lambda' passed incorrect first type! Got %s, expected %s",
          ltype_name(v->cell[0]->type), ltype_name(LVAL_QEXPR));
  LASSERT(v, v->cell[1]->type == LVAL_QEXPR,
          "Function 'lambda' passed incorrect second type! Got %s, expected %s",
          ltype_name(v->cell[1]->type), ltype_name(LVAL_QEXPR));

  for (unsigned i = 0; i < v->cell[0]->count; i++) {
    LASSERT(v, (v->cell[0]->cell[i]->type == LVAL_SYM),
            "cannot define non-symbol type %s",
            ltype_name(v->cell[0]->cell[i]->type));
  }

  lval *formals = lval_pop(v, 0);
  lval *body = lval_pop(v, 0);

  lval_free(v);
  return lval_lambda(formals, body);
}

lval *builtin_add(lenv *e, lval *a) { return builtin_op(e, a, "+"); }
lval *builtin_sub(lenv *e, lval *a) { return builtin_op(e, a, "-"); }
lval *builtin_mul(lenv *e, lval *a) { return builtin_op(e, a, "*"); }
lval *builtin_div(lenv *e, lval *a) { return builtin_op(e, a, "/"); }

lval *builtin_gt(lenv *e, lval *a) { return builtin_ord(e, a, ">"); }
lval *builtin_lt(lenv *e, lval *a) { return builtin_ord(e, a, "<"); }
lval *builtin_ge(lenv *e, lval *a) { return builtin_ord(e, a, ">="); }
lval *builtin_le(lenv *e, lval *a) { return builtin_ord(e, a, "<="); }

lval *builtin_load(lenv *e, lval *a) {
  LASSERT(a, a->count == 1,
          "'load' got an invalid amount of args. expected 1 got %u", a->count);
  LASSERT(a, a->cell[0]->type == LVAL_STR,
          "'load' can only load from string, got %s",
          ltype_name(a->cell[0]->type));

  mpc_result_t r;
  if (mpc_parse_contents(a->cell[0]->sym, Jlisp, &r)) {
    lval *expr = lval_read(r.output);
    mpc_ast_delete(r.output);

    while (expr->count) {
      lval *x = lval_eval(e, lval_pop(expr, 0));

      if (x->type == LVAL_ERR) {
        lval_print(x);
        putchar('\n');
      }

      lval_free(x);
    }

    lval_free(expr);
    lval_free(a);

    return lval_sexpr();
  } else {
    // parser error
    char *err_msg = mpc_err_string(r.error);
    mpc_err_delete(r.error);

    lval *err = lval_err("couldnt load lib: %s", err_msg);
    free(err_msg);
    lval_free(a);

    return err;
  }
}

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

  lval *result = lval_call(e, f, v);
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

lval *lval_read_str(mpc_ast_t *t) {
  t->contents[strlen(t->contents) - 1] = '\0';

  // skip first quote
  char *unescaped = malloc(strlen(t->contents + 1) + 1);
  strcpy(unescaped, t->contents + 1);
  unescaped = mpcf_unescape(unescaped);

  lval *str = lval_str(unescaped);
  free(unescaped);
  return str;
}

lval *lval_read(mpc_ast_t *t) {
  // since both number & symbol are terminal, we can just check for substring
  if (strstr(t->tag, "number"))
    return lval_read_num(t);
  if (strstr(t->tag, "symbol"))
    return lval_sym(t->contents);
  if (strstr(t->tag, "string"))
    return lval_read_str(t);

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

  for (int i = 0; i < t->children_num; i++) {
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
    if (strstr(t->children[i]->tag, "comment"))
      continue;
    v = lval_append(v, lval_read(t->children[i]));
  }
  return v;
}

void lenv_add_builtins(lenv *e) {
  lenv_add_builtin(e, "list", builtin_list);
  lenv_add_builtin(e, "head", builtin_head);
  lenv_add_builtin(e, "last", builtin_last);
  lenv_add_builtin(e, "tail", builtin_tail);
  lenv_add_builtin(e, "eval", builtin_eval);
  lenv_add_builtin(e, "join", builtin_join);

  // arithmetics
  lenv_add_builtin(e, "+", builtin_add);
  lenv_add_builtin(e, "-", builtin_sub);
  lenv_add_builtin(e, "*", builtin_mul);
  lenv_add_builtin(e, "/", builtin_div);

  // defs
  lenv_add_builtin(e, "def", builtin_def);
  lenv_add_builtin(e, "\\", builtin_lambda);
  lenv_add_builtin(e, "=", builtin_put);

  // comparisons
  lenv_add_builtin(e, "if", builtin_if);
  lenv_add_builtin(e, "==", builtin_eq);
  lenv_add_builtin(e, "!=", builtin_ne);
  lenv_add_builtin(e, ">", builtin_gt);
  lenv_add_builtin(e, ">=", builtin_ge);
  lenv_add_builtin(e, "<", builtin_lt);
  lenv_add_builtin(e, "<=", builtin_le);

  // utils
  lenv_add_builtin(e, "load", builtin_load);
  lenv_add_builtin(e, "error", builtin_error);
  lenv_add_builtin(e, "print", builtin_print);
}

int main(int argc, char **argv) {
  lenv *e = lenv_new();
  lenv_add_builtins(e);

  Number = mpc_new("number");
  Symbol = mpc_new("symbol");
  String = mpc_new("string");
  Sexpr = mpc_new("sexpr");
  Qexpr = mpc_new("qexpr");
  Expr = mpc_new("expr");
  Comment = mpc_new("comment");
  Jlisp = mpc_new("jlisp");

  mpca_lang(MPCA_LANG_DEFAULT, "                          \
      number  : /-?[0-9]+/ ;                              \
      symbol  : /[a-zA-Z0-9_+\\-*\\/\\\\=<>!&]+/ ;        \
      string  : /\"(\\\\.|[^\"])*\"/ ;                    \
      sexpr   : '(' <expr>* ')' ;                         \
      qexpr   : '{' <expr>* '}' ;                         \
      comment : /;[^\\r\\n]*/;                            \
      expr    : <number> | <symbol> | <string>            \
      | <sexpr> | <qexpr> | <comment>;                    \
      jlisp   : /^/ <expr>* /$/ ;                         \
    ",
            Number, Symbol, String, Sexpr, Qexpr, Comment, Expr, Jlisp);

  if (argc >= 2) {
    for (int i = 1; i < argc; i++) {
      lval *args = lval_append(lval_sexpr(), lval_str(argv[i]));
      lval *x = builtin_load(e, args);

      // if load is error return
      if (x->type == LVAL_ERR)
        lval_print(x);
      lval_free(x);
    }
  } else {

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
        mpc_err_print(r.error);
        mpc_err_delete(r.error);
      }

      free(input);
    }
  }

  mpc_cleanup(8, Number, Symbol, String, Sexpr, Qexpr, Comment, Expr, Jlisp);
  lenv_free(e);
  return 0;
}
