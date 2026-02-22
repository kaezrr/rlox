; Scopes
(block) @local.scope
(fun_declaration) @local.scope
(class_declaration) @local.scope

; Definitions
(var_declaration name: (identifier) @local.definition)
(fun_declaration name: (identifier) @local.definition)
(class_declaration name: (identifier) @local.definition)
(parameters (identifier) @local.definition)

; References
(identifier) @local.reference
