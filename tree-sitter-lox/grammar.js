/**
 * @file A dynamically typed programming language blending object-oriented and functional programming.
 * @author Anjishnu Banerjee <kaezr.dev@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "lox",

  word: ($) => $.identifier,

  extras: ($) => [/\s/, $.comment],

  supertypes: ($) => [$.declaration, $.statement, $.expression],

  inline: ($) => [$.expression],

  rules: {
    source_file: ($) => repeat($.declaration),

    block: ($) => seq("{", repeat($.declaration), "}"),

    // -------------------------------------------------------------------------
    // Declarations
    // -------------------------------------------------------------------------

    declaration: ($) =>
      choice(
        $.var_declaration,
        $.fun_declaration,
        $.class_declaration,
        $.statement,
      ),

    var_declaration: ($) =>
      seq(
        "var",
        field("name", $.identifier),
        optional(seq("=", field("value", $.expression))),
        ";",
      ),

    fun_declaration: ($) =>
      seq(
        "fun",
        field("name", $.identifier),
        "(",
        field("parameters", optional($.parameters)),
        ")",
        field("body", $.block),
      ),

    class_declaration: ($) =>
      seq(
        "class",
        field("name", $.identifier),
        optional(seq("<", field("superclass", $.identifier))),
        "{",
        repeat(field("member", $.member)),
        "}",
      ),

    member: ($) =>
      choice(
        seq(
          "class",
          field("name", $.identifier),
          "(",
          field("parameters", optional($.parameters)),
          ")",
          field("body", $.block),
        ),
        seq(
          field("name", $.identifier),
          "(",
          field("parameters", optional($.parameters)),
          ")",
          field("body", $.block),
        ),
        seq(field("name", $.identifier), field("body", $.block)),
      ),

    // -------------------------------------------------------------------------
    // Statements
    // -------------------------------------------------------------------------

    statement: ($) =>
      choice(
        $.expression_statement,
        $.if_statement,
        $.print_statement,
        $.return_statement,
        $.while_statement,
        $.for_statement,
        $.break_statement,
        $.block,
      ),

    expression_statement: ($) => seq($.expression, ";"),

    print_statement: ($) => seq("print", field("value", $.expression), ";"),

    return_statement: ($) =>
      seq("return", field("value", optional($.expression)), ";"),

    break_statement: (_) => seq("break", ";"),

    if_statement: ($) =>
      prec.right(
        seq(
          "if",
          "(",
          field("condition", $.expression),
          ")",
          field("consequence", $.statement),
          optional(seq("else", field("alternative", $.statement))),
        ),
      ),

    while_statement: ($) =>
      seq(
        "while",
        "(",
        field("condition", $.expression),
        ")",
        field("body", $.statement),
      ),

    for_statement: ($) =>
      seq(
        "for",
        "(",
        choice(
          field("initializer", $.var_declaration),
          field("initializer", $.expression_statement),
          ";",
        ),
        field("condition", optional($.expression)),
        ";",
        field("increment", optional($.expression)),
        ")",
        field("body", $.statement),
      ),

    // -------------------------------------------------------------------------
    // Expressions
    // -------------------------------------------------------------------------

    expression: ($) => $.assignment,

    assignment: ($) =>
      choice(
        seq(
          field("object", $.call),
          "[",
          field("index", $.expression),
          "]",
          "=",
          field("value", $.assignment),
        ),
        seq(
          field("object", $.call),
          ".",
          field("property", $.identifier),
          "=",
          field("value", $.assignment),
        ),
        seq(field("name", $.identifier), "=", field("value", $.assignment)),
        $.ternary,
      ),

    ternary: ($) =>
      seq(
        field("condition", $.logic_or),
        optional(
          seq(
            "?",
            field("consequence", $.ternary),
            ":",
            field("alternative", $.ternary),
          ),
        ),
      ),

    logic_or: ($) =>
      prec.left(1, seq($.logic_and, repeat(seq("or", $.logic_and)))),

    logic_and: ($) =>
      prec.left(2, seq($.equality, repeat(seq("and", $.equality)))),

    equality: ($) =>
      prec.left(
        3,
        seq($.comparison, repeat(seq(choice("!=", "=="), $.comparison))),
      ),

    comparison: ($) =>
      prec.left(
        4,
        seq($.term, repeat(seq(choice(">", ">=", "<", "<="), $.term))),
      ),

    term: ($) =>
      prec.left(5, seq($.factor, repeat(seq(choice("-", "+"), $.factor)))),

    factor: ($) =>
      prec.left(6, seq($.unary, repeat(seq(choice("/", "*", "%"), $.unary)))),

    unary: ($) => choice(prec.right(7, seq(choice("!", "-"), $.unary)), $.call),

    call: ($) =>
      prec.left(
        seq(
          field("callee", $.primary),
          repeat(
            choice(
              seq("(", field("arguments", optional($.arguments)), ")"),
              seq(".", field("property", $.identifier)),
              seq("[", field("index", $.expression), "]"),
            ),
          ),
        ),
      ),

    primary: ($) =>
      choice(
        $.number,
        $.string,
        "true",
        "false",
        "nil",
        "this",
        $.identifier,
        seq("super", ".", field("method", $.identifier)),
        seq("(", $.expression, ")"),
        seq(
          "fun",
          "(",
          field("parameters", optional($.parameters)),
          ")",
          field("body", $.block),
        ),
        seq(
          "[",
          optional(seq($.expression, repeat(seq(",", $.expression)))),
          "]",
        ),
      ),

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    parameters: ($) => seq($.identifier, repeat(seq(",", $.identifier))),

    arguments: ($) => seq($.expression, repeat(seq(",", $.expression))),

    // -------------------------------------------------------------------------
    // Terminals
    // -------------------------------------------------------------------------

    identifier: (_) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    number: (_) => /\d+(\.\d+)?/,

    string: (_) => /"[^"\\]*(?:\\.[^"\\]*)*"/,

    comment: (_) =>
      token(
        choice(seq("//", /.*/), seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")),
      ),
  },
});
