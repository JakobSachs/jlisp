(def [test-list] ["a" "b" "c"])

(print "Testing head and last symmetry:")
(print "--------------------------------")

(def [first-elem] (head test-list))
(def [last-elem] (last test-list))

(print "head result:")
(print first-elem)
(print "Expected: a (unwrapped)")

(print "")
(print "last result:")
(print last-elem)
(print "Expected: c (unwrapped)")

(print "")
(print "Both should now be unwrapped values, not lists!")

; Test that we can directly use head result without unwrapping
(print "")
(print "Direct usage test:")
(def [nums] [1 2 3])
(def [first-num] (head nums))
(print (+ first-num 10))
(print "Expected: 11")
