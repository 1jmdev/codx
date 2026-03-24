; Comments
(comment) @comment

; Keywords
[
  "as" "async" "await" "break" "case" "catch" "class" "const"
  "continue" "debugger" "default" "delete" "do" "else" "export"
  "extends" "finally" "for" "from" "if" "import" "in" "instanceof"
  "let" "new" "of" "static" "switch" "target" "throw"
  "try" "typeof" "var" "void" "while" "with" "yield"
] @keyword

["function"] @keyword.function
["return"] @keyword.return
["typeof" "instanceof" "in" "of" "delete" "void"] @keyword.operator

; Strings
(string) @string
(template_string) @string
(escape_sequence) @string.escape

; Numbers
(number) @number

; Booleans
(true) @boolean
(false) @boolean

; Null/Undefined
(null) @constant.builtin
(undefined) @constant.builtin

; Operators
[
  "+" "-" "*" "/" "%" "**" "++" "--"
  "==" "!=" "===" "!==" "<" ">" "<=" ">="
  "=" "+=" "-=" "*=" "/=" "%=" "**="
  "&&" "||" "??" "!" "~" "&" "|" "^" "<<" ">>" ">>>"
  "&&=" "||=" "??=" "&=" "|=" "^=" "<<=" ">>=" ">>>="
  "=>" "..." "??"
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_declaration name: (identifier) @function)
(method_definition name: (property_identifier) @function.method)
(call_expression function: (identifier) @function)
(call_expression function: (member_expression property: (property_identifier) @function.method))

; Types/Classes
(class_declaration name: (identifier) @type)
(new_expression constructor: (identifier) @type)

; Variables
(identifier) @variable
(super) @variable.builtin
(shorthand_property_identifier_pattern) @variable.parameter

; Properties
(property_identifier) @property
(shorthand_property_identifier) @property

; Constants
(identifier) @constant
