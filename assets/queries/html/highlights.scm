; DOCTYPE
(doctype) @keyword

; Tags
(tag_name) @function
(end_tag (tag_name) @function)

; Attributes
(attribute_name) @attribute
(attribute_value) @string
(quoted_attribute_value) @string

; Comments
(comment) @comment

; Punctuation
["<" ">" "</" "/>" "<!" "="] @punctuation.bracket
["="] @operator

; Scripts and Styles
(script_element) @special
(style_element) @special

; Entities
(entity) @escape

; Text
(text) @variable
