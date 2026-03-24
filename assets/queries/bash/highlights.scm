; Comments
(comment) @comment

; Keywords
[
  "case" "do" "done" "elif" "else" "esac" "fi" "for"
  "function" "if" "in" "select" "then" "until" "while"
] @keyword

["return"] @keyword.return

; Strings
(string) @string
(raw_string) @string
(ansi_c_string) @string
(string_expansion) @string
(escape_sequence) @string.escape

; Numbers
(number) @number

; Variables
(variable_name) @variable
(special_variable_name) @variable.builtin
(positional) @variable.builtin

; Functions
(function_definition name: (word) @function)
(command name: (word) @function)
(command name: (command_name (word) @function))

; Operators
[";" "&" "|" "||" "&&" ">>" ">" "<" "<<" "<<<" ">>" ">&" "<&"] @operator
["=" "+=" "-=" "*=" "/="] @operator

; Punctuation
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Heredoc
(heredoc_body) @string
(heredoc_start) @keyword
