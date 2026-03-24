; Comments
(comment) @comment

; Keywords
[
  "case" "do" "done" "elif" "else" "esac" "fi" "for"
  "function" "if" "in" "select" "then" "until" "while"
] @keyword

; Strings
(string) @string
(raw_string) @string
(ansi_c_string) @string

; Numbers
(number) @number

; Variables
(variable_name) @variable
(special_variable_name) @variable.builtin

; Functions
(function_definition name: (word) @function)
(command name: (command_name (word) @function))

; Operators
[";" "&" "|" "||" "&&" ">>" ">" "<" "<<" "<<<" ">>" ">&" "<&"] @operator
["=" "+=" "-=" "*=" "/="] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Heredoc
(heredoc_body) @string
(heredoc_start) @keyword
