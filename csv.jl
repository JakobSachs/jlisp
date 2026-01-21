; Proper CSV Parser - Returns 2D array (list of lists)

; Helper function with accumulator for tail recursion
; Takes lines to process and accumulated result
(fun [parse-csv-lines lines acc]
  [if (== (len lines) 0)
    [acc]
    [(parse-csv-lines
       (tail lines)
       (join acc (list (split ',' (head lines)))))]]
)

; Main CSV parser function
; Takes a filepath, returns 2D array (list of lists)
(fun [parse-csv filepath]
  (parse-csv-lines (split '\n' (read filepath)) [])
)

; Test it
(print "Parsing example.csv into 2D array...")
(def [data] (parse-csv "example.csv"))

(print "")
(print "Result (2D array):")
(print data)

(print "")
(print "Number of rows:")
(print (len data))

(print "")
(print "Row 0 (header):")
(print (head data))

(print "")
(print "Row 1:")
(print (head (tail data)))

(print "")
(print "Row 2:")
(print (head (tail (tail data))))
