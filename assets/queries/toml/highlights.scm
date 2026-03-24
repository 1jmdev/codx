; Comments
(comment) @comment

; Keys
(bare_key) @property
(quoted_key) @property

; Tables
(table (bare_key) @namespace)

; Strings
(string) @string
(escape_sequence) @string.escape

; Numbers
(integer) @number
(float) @float

; Booleans
(boolean) @boolean

; Dates
(offset_date_time) @special
(local_date_time) @special
(local_date) @special
(local_time) @special

; Punctuation
["," "="] @punctuation.delimiter
["[" "]" "{" "}"] @punctuation.bracket
["[[" "]]"] @punctuation.bracket
