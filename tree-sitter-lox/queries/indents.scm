; Indent after opening braces
(block "{" @indent.begin)
(block "}" @indent.end)

; Indent inside class body
(class_declaration "{" @indent.begin)
(class_declaration "}" @indent.end)

; Indent inside if/else/while/for bodies that aren't blocks
(if_statement
  consequence: (_) @indent.begin
  (#not-kind-eq? @indent.begin "block"))

(if_statement
  alternative: (_) @indent.begin
  (#not-kind-eq? @indent.begin "block"))

(while_statement
  body: (_) @indent.begin
  (#not-kind-eq? @indent.begin "block"))

(for_statement
  body: (_) @indent.begin
  (#not-kind-eq? @indent.begin "block"))

; Align closing braces with opening line
(block "}" @indent.branch)
(class_declaration "}" @indent.branch)
