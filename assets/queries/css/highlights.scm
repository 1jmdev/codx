; Comments
(comment) @comment

; Selectors
(tag_name) @type
(class_name) @function
(id_name) @constant
(universal_selector) @keyword
(pseudo_class_selector (class_name) @keyword)
(pseudo_element_selector (tag_name) @keyword)
(attribute_selector (attribute_name) @attribute)

; Properties
(property_name) @property
(feature_name) @property

; Values
(plain_value) @variable
(string_value) @string
(color_value) @constant
(integer_value) @number
(float_value) @float
(unit) @keyword.type

; Keywords
["@media" "@keyframes" "@import" "@charset" "@namespace" "@supports" "@layer"] @keyword
["from" "to" "not" "and" "or" "only" "selector"] @keyword

; Operators
["," ";" ":"] @punctuation.delimiter
["{" "}"] @punctuation.bracket
["(" ")"] @punctuation.bracket

; Functions
(function_name) @function
(call_expression function: (function_name) @function)
