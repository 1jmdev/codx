; Comments
(comment) @comment

; Keywords
[
  "abstract" "as" "async" "await" "break" "case" "catch" "class" "const"
  "continue" "debugger" "declare" "default" "delete" "do" "else" "enum"
  "export" "extends" "finally" "for" "from" "if" "implements" "import"
  "in" "infer" "instanceof" "interface" "keyof" "let" "module" "namespace"
  "new" "of" "override" "readonly" "satisfies" "static" "switch" "target"
  "this" "throw" "try" "type" "typeof" "unique" "var" "void" "while"
  "with" "yield"
] @keyword

["function"] @keyword.function
["return"] @keyword.return
["typeof" "instanceof" "in" "of" "delete" "void" "keyof" "infer"] @keyword.operator

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
  "=>" "..." "?." "??" "!"
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_declaration name: (identifier) @function)
(function name: (identifier) @function)
(method_definition name: (property_identifier) @function.method)
(call_expression function: (identifier) @function)
(call_expression function: (member_expression property: (property_identifier) @function.method))

; Types
(type_identifier) @type
(predefined_type) @type.builtin
(class_declaration name: (type_identifier) @type)
(interface_declaration name: (type_identifier) @type)
(type_alias_declaration name: (type_identifier) @type)

; Variables
(identifier) @variable
(this) @variable.builtin

; Properties
(property_identifier) @property

; Decorators
(decorator) @attribute
