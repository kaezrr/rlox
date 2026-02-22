# Lox Language Reference

This is the language reference for Lox. For a general introduction to Lox, see [Crafting Interpreters](https://craftinginterpreters.com/contents.html) by Robert Nystrom. This document focuses on the specifics of this implementation and its extensions beyond the book.

## Table of Contents

- [Types](#types)
- [Operators](#operators)
- [Grammar](#grammar)
- [Statements](#statements)
- [Functions](#functions)
- [Classes](#classes)
- [Lists](#lists)
- [Native Functions](#native-functions)
- [Differences from the Book](#differences-from-the-book)

## Types

| Type    | Description                      | Example            |
| ------- | -------------------------------- | ------------------ |
| number  | 64-bit floating point            | `42`, `3.14`       |
| string  | UTF-8 string                     | `"hello"`          |
| boolean | `true` or `false`                | `true`             |
| nil     | Absence of a value               | `nil`              |
| list    | Ordered collection of any values | `[1, "two", true]` |

## Operators

### Comments

```lox
// This is a single line comment

/* This is a
   multiline comment */
```

### Arithmetic

| Operator    | Description               |
| ----------- | ------------------------- |
| `+`         | Addition or string concat |
| `-`         | Subtraction               |
| `*`         | Multiplication            |
| `/`         | Division                  |
| `%`         | Modulo                    |
| `-` (unary) | Negation                  |

### Comparison

| Operator | Description      |
| -------- | ---------------- |
| `==`     | Equal            |
| `!=`     | Not equal        |
| `<`      | Less than        |
| `<=`     | Less or equal    |
| `>`      | Greater than     |
| `>=`     | Greater or equal |

### Logical

| Operator | Description |
| -------- | ----------- |
| `and`    | Logical and |
| `or`     | Logical or  |
| `!`      | Logical not |

### Operator Overloading

The `+` operator supports mixed string and number operands by coercing the number to its string representation:

| Left   | Right  | Result               |
| ------ | ------ | -------------------- |
| number | number | numeric addition     |
| string | string | string concatenation |
| string | number | string concatenation |
| number | string | string concatenation |

The comparison operators `<`, `<=`, `>`, `>=` work on both numbers (numeric order) and strings (lexicographic order), but mixing the two types is a runtime error.

### Truthiness

Lox follows Ruby's rule: `false` and `nil` are falsy, and everything else is truthy, including `0` and empty strings.

```lox
if (0)   print "truthy"; // truthy
if ("")  print "truthy"; // truthy
if (nil) print "truthy"; // not printed
```

### Ternary

```lox
var result = condition ? then_value : else_value;
```

## Grammar

```
program        -> declaration* EOF ;

declaration    -> varDecl | funDecl | classDecl | statement ;

varDecl        -> "var" IDENTIFIER ( "=" expression )? ";" ;
funDecl        -> "fun" IDENTIFIER "(" parameters? ")" block ;   // sugar for: var IDENTIFIER = fun(...) block
classDecl      -> "class" IDENTIFIER ( "<" IDENTIFIER )? "{" member* "}" ;
member         -> "class" IDENTIFIER "(" parameters? ")" block   // static method
               | IDENTIFIER "(" parameters? ")" block           // method
               | IDENTIFIER block ;                             // getter (no parentheses)

statement      -> exprStmt
               | ifStmt
               | printStmt
               | returnStmt
               | whileStmt
               | forStmt
               | breakStmt
               | block ;

exprStmt       -> expression ";" ;
printStmt      -> "print" expression ";" ;
returnStmt     -> "return" expression? ";" ;
breakStmt      -> "break" ";" ;
block          -> "{" declaration* "}" ;

ifStmt         -> "if" "(" expression ")" statement ( "else" statement )? ;
whileStmt      -> "while" "(" expression ")" statement ;
forStmt        -> "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;

expression     -> assignment ;
assignment     -> call "[" expression "]" "=" assignment
               | call "." IDENTIFIER "=" assignment
               | IDENTIFIER "=" assignment
               | ternary ;
ternary        -> logic_or ( "?" ternary ":" ternary )? ;
logic_or       -> logic_and ( "or" logic_and )* ;
logic_and      -> equality ( "and" equality )* ;
equality       -> comparison ( ( "!=" | "==" ) comparison )* ;
comparison     -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           -> factor ( ( "-" | "+" ) factor )* ;
factor         -> unary ( ( "/" | "*" | "%" ) unary )* ;
unary          -> ( "!" | "-" ) unary | call ;
call           -> primary ( "(" arguments? ")" | "." IDENTIFIER | "[" expression "]" )* ;
primary        -> NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")"
               | "this"
               | "super" "." IDENTIFIER
               | IDENTIFIER
               | "fun" "(" parameters? ")" block
               | "[" ( expression ( "," expression )* )? "]" ;

parameters     -> IDENTIFIER ( "," IDENTIFIER )* ;
arguments      -> expression ( "," expression )* ;
```

## Statements

### Variables

```lox
var x = 10;
var name = "Lox";
var nothing; // uninitialized
```

Using an uninitialized variable and unused local variables are both **semantic errors**.

### If / Else

```lox
if (condition) {
    // ...
} else {
    // ...
}
```

### While

```lox
while (condition) {
    // ...
}
```

### For

```lox
for (var i = 0; i < 10; i = i + 1) {
    // ...
}
```

### Break

```lox
while (true) {
    if (done) break;
}
```

### Print

```lox
print "hello world";
```

## Functions

Functions are first-class values in Lox.

```lox
fun greet(name) {
    return "Hello, " + name + "!";
}
print greet("Rex"); // Hello, Rex!
```

### Lambdas

Anonymous functions can be created with `fun` without a name:

```lox
var double = fun(x) { return x * 2; };
print double(5); // 10
```

### Closures

Functions close over their surrounding scope:

```lox
fun makeCounter() {
    var count = 0;
    return fun() {
        count = count + 1;
        return count;
    };
}
var counter = makeCounter();
print counter(); // 1
print counter(); // 2
```

## Classes

```lox
class Animal {
    init(name) {
        this.name = name;
    }

    speak() {
        print this.name + " makes a sound.";
    }
}
```

### Inheritance

```lox
class Dog < Animal {
    speak() {
        super.speak();
        print this.name + " barks!";
    }
}
```

### Static Methods

Static methods are declared with the `class` keyword and called on the class itself, not on instances. They do not have access to `this`.

```lox
class MathUtils {
    class square(x) {
        return x * x;
    }
}
print MathUtils.square(4); // 16
```

### Getters

Getters are declared like methods but without parentheses, and are accessed like properties with no call needed:

```lox
class Circle {
    init(radius) {
        this.radius = radius;
    }

    area {
        return 3.14159 * this.radius * this.radius;
    }
}

var c = Circle(5);
print c.area; // 78.53...
```

## Lists

Lists are ordered collections that can hold any type.

```lox
var nums = [1, 2, 3];
var mixed = [42, "hello", true, nil];
var empty = [];
```

### Indexing

```lox
print nums[0]; // 1
nums[1] = 99;
```

Indices must be non-negative integers. Out of bounds access is a **runtime error**.

### Nested Lists

```lox
var matrix = [[1, 2], [3, 4]];
print matrix[0][1]; // 2
```

## Native Functions

| Function | Signature           | Description                                          |
| -------- | ------------------- | ---------------------------------------------------- |
| `clock`  | `clock()`           | Returns elapsed time in seconds since the Unix epoch |
| `input`  | `input()`           | Reads a line from stdin as a string                  |
| `number` | `number(value)`     | Converts a value to a number, runtime error if fails |
| `push`   | `push(list, value)` | Appends a value to the end of a list                 |
| `pop`    | `pop(list)`         | Removes and returns the last element, nil if empty   |
| `len`    | `len(list)`         | Returns the length of a list                         |

Redefining native functions is a **semantic error**.

## Differences from the Book

- **Modulo operator** - added `%` for remainder division.
- **Lists** - added list literals, indexing, and index assignment.
- **Lambdas** - anonymous functions using `fun(params) { ... }` syntax.
- **Function declarations** - `fun name() {}` is syntax sugar for `var name = fun() {}`.
- **Ternary operator** - `condition ? then : else`.
- **Break statement** - `break` inside loops.
- **Multi-line comments** - added comments that can span multiple lines.
- **Static methods** - declared with the `class` keyword inside a class body, called on the class itself, no access to `this`.
- **Getters** - methods declared without parentheses, accessed as properties.
- **Native functions** - added `input`, `number`, `push`, `pop`, `len` beyond the book's `clock`.
- **Unused variables** - unused local variables are a semantic error.
- **Uninitialized variables** - using a variable before it is assigned is a semantic error.
- **Native redefinition** - redefining native functions is a semantic error.
