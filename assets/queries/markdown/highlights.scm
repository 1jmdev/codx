; Headings
(atx_heading (atx_h1_marker) @keyword)
(atx_heading (atx_h2_marker) @keyword)
(atx_heading (atx_h3_marker) @keyword)
(atx_heading (atx_h4_marker) @keyword)
(atx_heading (atx_h5_marker) @keyword)
(atx_heading (atx_h6_marker) @keyword)

; Code blocks
(fenced_code_block) @string
(fenced_code_block_delimiter) @punctuation.bracket
(code_span) @string

; Links and images
(link_text) @function
(link_destination) @string
(image_description) @function

; Emphasis
(emphasis) @special
(strong_emphasis) @special

; Block elements
(block_quote) @string
(thematic_break) @keyword

; Lists
(list_marker_minus) @punctuation.bracket
(list_marker_plus) @punctuation.bracket
(list_marker_star) @punctuation.bracket
(list_marker_dot) @punctuation.bracket
(list_marker_parenthesis) @punctuation.bracket

; HTML
(html_tag) @attribute

; Inline elements
(inline_link) @special
(full_reference_link) @special
(collapsed_reference_link) @special

; Escape
(backslash_escape) @escape
