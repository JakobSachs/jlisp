# jlisp

My own lisp that i built, based on Daniel Holdens "Build your own lisp"

I built this partially to learn some more old-fashion C, but also to really understand what a lisp is.

I also built all of this on my IBM Thinkpad T34 with the goal of using it for this years AdventOfCode.

## Features i want to add:

- add the base-lib as an automatic import
- COLORS!!!!!!
- better errors: once theres an error, trace it back through the callgraph, so we can get something more meaningful then `error: Function 'head' passed {}!`
- more builtins: spirit of the book/language was to do alot of the advanced functions in the lisp itself, but im quite resource restrained, so i want as much as possible to be compiled
- rewrite it in rust (duh) 
    - C is fun, but god its so unergonomic (in the style the book teaches it)

