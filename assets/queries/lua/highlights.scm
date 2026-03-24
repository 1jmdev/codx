; Comments
(comment) @comment

; Keywords
[
  "and" "do" "else" "elseif" "end"
  "for" "goto" "if" "in" "local" "not" "or"
  "repeat" "then" "until" "while"
] @keyword

["function"] @keyword.function
["return"] @keyword.return
["not" "and" "or"] @keyword.operator

; Strings
(string) @string
(escape_sequence) @string.escape

; Numbers
(number) @number

; Booleans
(true) @boolean
(false) @boolean

; Nil
(nil) @constant.builtin

; Operators
[
  "+" "-" "*" "/" "//" "%" "^" "&" "|" "~" "<<" ">>"
  "==" "~=" "<" ">" "<=" ">="
  "=" ".." "#"
  "~="
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["." ".."] @punctuation

; Functions
(function_declaration name: (identifier) @function)
(function_call name: (identifier) @function)
(method_index_expression method: (identifier) @function.method)

; Variables
(identifier) @variable

; Fields

; Labels
(label_statement (identifier) @label)
