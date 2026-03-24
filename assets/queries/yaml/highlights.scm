; Comments
(comment) @comment

; Keys
(block_mapping_pair key: (flow_node (plain_scalar (string_scalar) @property)))
(block_mapping_pair key: (flow_node (double_quote_scalar) @property))
(block_mapping_pair key: (flow_node (single_quote_scalar) @property))

; Strings
(double_quote_scalar) @string
(single_quote_scalar) @string
(block_scalar) @string

; Scalars
(plain_scalar (string_scalar) @variable)
(plain_scalar (boolean_scalar) @boolean)
(plain_scalar (null_scalar) @constant.builtin)
(plain_scalar (integer_scalar) @number)
(plain_scalar (float_scalar) @float)

; Anchors and Aliases
(anchor_name) @label
(alias_name) @label

; Punctuation
[":" "-" ","] @punctuation.delimiter
["{" "}" "[" "]"] @punctuation.bracket

; Tags
(tag) @keyword.type

; Documents
["---" "..."] @keyword
