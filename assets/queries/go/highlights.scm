; Comments
(comment) @comment

; Keywords
[
  "break" "case" "chan" "const" "continue" "default" "defer"
  "else" "fallthrough" "for" "go" "goto" "if" "import"
  "interface" "map" "package" "range" "select" "struct"
  "switch" "type" "var"
] @keyword

["func"] @keyword.function
["return"] @keyword.return

; Strings
(interpreted_string_literal) @string
(raw_string_literal) @string
(escape_sequence) @string.escape
(rune_literal) @string

; Numbers
(int_literal) @number
(float_literal) @float
(imaginary_literal) @number

; Booleans
(true) @boolean
(false) @boolean

; Nil
(nil) @constant.builtin

; Operators
[
  "+" "-" "*" "/" "%" "&" "|" "^" "<<" ">>" "&^"
  "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "&^="
  "==" "!=" "<" ">" "<=" ">="
  "=" ":=" "++" "--" "!" "&&" "||" "<-" "..."
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_declaration name: (identifier) @function)
(method_declaration name: (field_identifier) @function.method)
(call_expression function: (identifier) @function)
(call_expression function: (selector_expression field: (field_identifier) @function.method))

; Types
(type_identifier) @type
(package_identifier) @namespace

; Variables
(identifier) @variable

; Parameters
(parameter_declaration (identifier) @variable.parameter)

; Constants
(const_spec name: (identifier) @constant)
