(def [nil] [])
(def [true] [1])
(def [false] [0])

;; basic func def


(def [unpack] (\ [f l] 
  [eval (join (list f) l)]
))

(def [pack] (\ [f & xs] [f xs]))

(def [curry] unpack)
(def [uncurry] pack)

(def [do] (\ [& l] [
  if (== l nil)
    [nil]
    [last l]
]))

(def [let] (\ [b] [
  ((\[_] b) () )
]))

; misc
(def [flip] (\ [f a b] [f b a]))
(def [ghost] (\ [& xs] [eval xs])) 
(def [comp] (\ [f g x] [f (g x)])) 

; list accessors
(def [first] (\ [l] [eval (head l)]))
(def [second] (\ [l] [eval (head (tail l))]))
(def [third] (\ [l] [eval (head (tail (tail l)))]))
(def [nth] (\ [n l] [
  if (== n 0)
    [first l]
    [nth (- n 1) (tail l)]
]))


(def [take] (\ [n l] [
  if (== n 0)
    [nil]
    [join (head l) (take (- n 1) (tail l))]
]))


(def [drop] (\ [n l] [
  if (== n 0)
    [l]
    [drop (- n 1) (tail l)]
]))

(def [splitn] (\ [n l] [
  list [take n l] [drop n l]
]))

(def [contains] (\ [x l] [
  if (== l nil)
    [false]
    [if (== x (first l))
      [true]
      [contains x (tail l)]
    ]
]))

; basic functional list funcs

(def [map] (\ [f l] [
  if (== l nil)
    [nil]
    [join (list(f (first l))) (map f(tail l))]
]))

(def [filter] (\ [p l] [
  if (== l nil)
    [nil]
    [join (if (p (first l)) [head l] [nil])
      (filter p (tail l))]
]))

(def [foldl] (\ [f i l] [
  if (== l nil)
    [i]
    [foldl f (f i (first l)) (tail l)]
]))

(def [sum] (\ [l] [foldl + 0 l]))
(def [product] (\ [l] [foldl * 1 l]))

; control-flow
(def [select] (\ [& cs] [
  if (== cs nil)
    [error "No selection found"]
    [if (first (first cs))
      [second (first cs)]
      [unpack select (tail cs)]
    ]  
]))
(def [case] (\ [x & cs] [
  if (== cs nil)
    [error "No case found"]
    [if (== x (first (first cs)))
      [second (first cs)]
      [unpack case (join (list x) (tail cs)) ]
    ]  
]))


