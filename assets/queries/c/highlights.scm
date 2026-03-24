; Comments
(comment) @comment

; Keywords
[
  "break" "case" "const" "continue" "default" "do" "else"
  "enum" "extern" "for" "goto" "if" "inline" "register"
  "restrict" "return" "sizeof" "static" "struct" "switch"
  "typedef" "union" "volatile" "while" "_Alignas" "_Alignof"
  "_Atomic" "_Generic" "_Noreturn"
] @keyword

["return"] @keyword.return
["sizeof"] @keyword.operator

; Strings
(string_literal) @string
(escape_sequence) @string.escape
(char_literal) @string

; Numbers
(number_literal) @number

; Booleans
(true) @boolean
(false) @boolean

; Null
(null) @constant.builtin

; Operators
[
  "+" "-" "*" "/" "%" "++" "--"
  "==" "!=" "<" ">" "<=" ">="
  "=" "+=" "-=" "*=" "/=" "%="
  "&&" "||" "!" "~" "&" "|" "^" "<<" ">>"
  "&=" "|=" "^=" "<<=" ">>="
  "->" "." "..." "?"
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_definition declarator: (function_declarator declarator: (identifier) @function))
(declaration declarator: (function_declarator declarator: (identifier) @function))
(call_expression function: (identifier) @function)

; Types
(type_identifier) @type
(primitive_type) @type.builtin

; Variables
(identifier) @variable

; Preprocessor
(preproc_include) @keyword.storage
(preproc_ifdef) @keyword.storage
