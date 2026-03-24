; Comments
(line_comment) @comment
(block_comment) @comment
(doc_comment) @comment.doc

; Keywords
[
  "as" "async" "await" "break" "const" "continue" "crate"
  "dyn" "else" "enum" "extern" "for" "if" "impl" "in"
  "let" "loop" "match" "mod" "move" "mut" "pub" "ref"
  "self" "Self" "static" "struct" "super" "trait" "type"
  "unsafe" "use" "where" "while"
] @keyword

["fn"] @keyword.function
["return"] @keyword.return

; Strings
(string_literal) @string
(raw_string_literal) @string
(char_literal) @string
(escape_sequence) @string.escape

; Numbers
(integer_literal) @number
(float_literal) @float

; Booleans
(boolean_literal) @boolean

; Operators
[
  "+" "-" "*" "/" "%" "^" "&" "|" "<<" ">>"
  "+=" "-=" "*=" "/=" "%=" "^=" "&=" "|=" "<<=" ">>="
  "==" "!=" "<" ">" "<=" ">="
  "=" "!" "&&" "||" ".." "..=" "->" "=>" "::" "@"
] @operator

; Punctuation
["," ";" ":"] @punctuation.delimiter
["(" ")" "[" "]" "{" "}"] @punctuation.bracket

; Functions
(function_item name: (identifier) @function)
(function_signature_item name: (identifier) @function)
(call_expression function: (identifier) @function)
(call_expression function: (field_expression field: (field_identifier) @function.method))
(macro_invocation macro: (identifier) @function.macro)

; Types
(type_identifier) @type
(primitive_type) @type.builtin

; Variables
(identifier) @variable
(self) @variable.builtin
(parameter pattern: (identifier) @variable.parameter)

; Constants
(const_item name: (identifier) @constant)
(enum_variant name: (identifier) @constant)

; Attributes
(attribute_item) @attribute
(inner_attribute_item) @attribute

; Lifetimes
(lifetime_identifier) @label

; Namespaces
(use_declaration argument: (scoped_identifier path: (identifier) @namespace))
(scoped_identifier path: (identifier) @namespace)
