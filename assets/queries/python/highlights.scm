; Comments
(comment) @comment

; Keywords
[
  "and" "as" "assert" "async" "await" "break" "class" "continue"
  "del" "elif" "else" "except" "exec" "finally" "for" "from"
  "global" "if" "import" "in" "is" "lambda" "nonlocal" "not"
  "or" "pass" "print" "raise" "try" "while" "with" "yield"
] @keyword

["def"] @keyword.function
["return" "yield"] @keyword.return
["not" "and" "or" "is" "in" "lambda"] @keyword.operator

; Strings
(string) @string
(escape_sequence) @string.escape
(interpolation) @special

; Numbers
(integer) @number
(float) @float

; Booleans
(true) @boolean
(false) @boolean

; None
(none) @constant.builtin

; Operators
[
  "+" "-" "*" "/" "//" "%" "**" "@"
  "+=" "-=" "*=" "/=" "//=" "%=" "**=" "@="
  "==" "!=" "<" ">" "<=" ">="
  "=" "|" "&" "^" "~" "<<" ">>"
  "|=" "&=" "^=" "<<=" ">>="
  "->" ":=" "..."
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_definition name: (identifier) @function)
(call function: (identifier) @function)
(call function: (attribute attribute: (identifier) @function.method))
(decorator) @attribute

; Classes
(class_definition name: (identifier) @type)

; Variables
(identifier) @variable
(self_parameter) @variable.builtin

; Parameters
(parameters (identifier) @variable.parameter)
(default_parameter name: (identifier) @variable.parameter)
(typed_parameter (identifier) @variable.parameter)

; Constants
((identifier) @constant)

; Builtins
((identifier) @function.builtin)
