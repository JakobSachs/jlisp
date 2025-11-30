(def {nil} {})
(def {true} {1})
(def {false} {0})

;; basic func def
(def {fun} (\ {f b} {
  def (head f) (\ (tail f) b)
 }))

(fun {unpack f l} {
  eval (join (list f) l)
})

(fun {pack f & xs} {f xs})

(def {curry} unpack)
(def {uncurry} pack)

(fun {do & l} {
  if (== l nil)
    {nil}
    {last l}
})

(fun {let b} {
  ((\{_} b) () )
})

; logical ops
(fun {not x} {- 1 x})
(fun {or x y} {+ x y})
(fun {and x y} {* x y})

; misc
(fun {flip f a b} {f b a})
(fun {ghost &xs} {eval xs}) 
(fun {comp f g x} {f (g x)}) 

; list accessors
(fun {first l} {eval (head l)})
(fun {second l} {eval (head (tail l))})
(fun {third l} {eval (head (tail (tail l)))})
(fun {nth n l} {
  if (== n 0)
    {first l}
    {nth (- n 1) (tail l)}
})


(fun {take n l} {
  if (== n 0)
    {nil}
    {join (head l) (take (- n 1) (tail l))}
})


(fun {drop n l} {
  if (== n 0)
    {l}
    {drop (- n 1) (tail l)}
})

(fun {splitn n l} {
  list {take n l} {drop n l}
})

(fun {contains x l} {
  if (== l nil)
    {false}
    {if (== x (first l))
      {true}
      {contains x (tail l)}
    }
})

; basic functional list funcs

(fun {map f l} {
  if (== l nil)
    {nil}
    {join (list(f (first l))) (map f(tail l))}
})

(fun {filter p l} {
  if (== l nil)
    {nil}
    {join (if (p (first l)) {head l} {nil})
      (filter p (tail l))}
})

(fun {foldl f i l} {
  if (== l nil)
    {i}
    {foldl f (f i (first l)) (tail l)}
})

(fun {sum l} {foldl + 0 l})
(fun {product l} {foldl * 1 l})

; control-flow
(fun {select & cs} {
  if (== cs nil)
    {error "No selection found"}
    {if (first (first cs))
      {second (first cs)}
      {unpack select (tail cs)}
    }  
})
(fun {case x & cs} {
  if (== cs nil)
    {error "No case found"}
    {if (== x (first (first cs)))
      {second (first cs)}
      {unpack case (join (list x) (tail cs)) }
    }  
})


(print (sum (range 5000)))
