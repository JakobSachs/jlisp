; Test that variable names can now contain digits
(def [x1] 42)
(print x1)

(def [row2] "hello")
(print row2)

(def [data123] (list 1 2 3))
(print data123)

; Test that negative numbers still work
(def [neg] -456)
(print neg)

; Test arithmetic with digit-named variables
(def [sum1] (+ x1 10))
(print sum1)
