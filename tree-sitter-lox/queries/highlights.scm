(number) @number
(string) @string
(comment) @comment

"var" @keyword
"fun" @keyword.function
"class" @keyword
"return" @keyword.return
"print" @keyword
"if" @keyword.conditional
"else" @keyword.conditional
"while" @keyword.repeat
"for" @keyword.repeat
"break" @keyword.repeat
"and" @keyword.operator
"or" @keyword.operator
"true" @boolean
"false" @boolean
"nil" @constant.builtin
"this" @variable.builtin
"super" @variable.builtin

((call callee: (primary (identifier) @function.builtin))
 (#match? @function.builtin "^(clock|input|number|push|pop|len)$"))
(fun_declaration name: (identifier) @function)
(class_declaration name: (identifier) @type)
(class_declaration superclass: (identifier) @type)
(member name: (identifier) @function.method)
(parameters (identifier) @variable.parameter)
(call callee: (primary (identifier) @function.call))
(call property: (identifier) @function.method)
(identifier) @variable

"+" @operator  "-" @operator  "*" @operator  "/" @operator
"%" @operator  "=" @operator  "==" @operator  "!=" @operator
"<" @operator  "<=" @operator  ">" @operator  ">=" @operator
"!" @operator  "?" @operator  ":" @operator

"(" @punctuation.bracket  ")" @punctuation.bracket
"{" @punctuation.bracket  "}" @punctuation.bracket
"[" @punctuation.bracket  "]" @punctuation.bracket
"," @punctuation.delimiter  ";" @punctuation.delimiter
"." @punctuation.delimiter
