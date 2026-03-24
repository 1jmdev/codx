; Comments
(comment) @comment

; Keywords
[
  "break" "case" "catch" "class" "const" "consteval" "constexpr"
  "constinit" "continue" "co_await" "co_return" "co_yield"
  "default" "delete" "do" "else" "enum" "explicit" "export"
  "extern" "final" "for" "friend" "goto" "if" "import" "inline"
  "module" "mutable" "namespace" "new" "noexcept" "operator"
  "override" "private" "protected" "public" "register" "requires"
  "sizeof" "static" "static_assert" "struct" "switch" "template"
  "this" "throw" "try" "typedef" "typename" "union"
  "using" "virtual" "volatile" "while"
] @keyword

["return" "co_return"] @keyword.return
["new" "delete" "sizeof" "typeof" "alignof" "decltype"] @keyword.operator
["class" "struct" "enum" "namespace" "template"] @keyword.type

; Strings
(string_literal) @string
(raw_string_literal) @string
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
  "->" "." "..." "?" ":"
  "::" "->*" ".*"
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_definition declarator: (function_declarator declarator: (identifier) @function))
(function_definition declarator: (function_declarator declarator: (qualified_identifier name: (identifier) @function)))
(call_expression function: (identifier) @function)
(call_expression function: (qualified_identifier name: (identifier) @function))

; Types
(type_identifier) @type
(primitive_type) @type.builtin
(namespace_identifier) @namespace

; Variables
(identifier) @variable
(this) @variable.builtin

; Templates
(template_type name: (type_identifier) @type)
