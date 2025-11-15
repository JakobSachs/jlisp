; basic func def
(def {fun} (\ {f b} {
  def (head f) (\ (tail f) b)
 }))

(fun {unpack f l} {
  eval (join (list f) l)
})

(fun {pack f & xs} {f xs})

(def {curry} unpack)
(def {uncurry} pack)
