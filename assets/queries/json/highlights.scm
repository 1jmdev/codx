; Strings
(string) @string
(escape_sequence) @string.escape

; Numbers
(number) @number

; Booleans
(true) @boolean
(false) @boolean

; Null
(null) @constant.builtin

; Object keys
(pair key: (string) @property)

; Punctuation
["," ":"] @punctuation.delimiter
["{" "}"] @punctuation.bracket
["[" "]"] @punctuation.bracket
