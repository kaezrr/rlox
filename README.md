## Lox

Two implementations of the Lox Language:

- `rlox` -> A tree-walk interpreter written in Rust
- `zlox` -> A bytecode virtual machine in Zig

An educational project made with the intention of learning about compilers and programming languages.

**Referred books:** [Crafting Interpreters](https://craftinginterpreters.com/contents.html) by Robert Nystrom

## Features

- Dynamically typed
- Closures and first-class functions
- Classes with inheritance
- Dynamic lists with indexing
- Lambda expressions
- Static class methods and getters
- Ternary and loops and conditionals
- Number conversion and other helpful native functions

## Examples

### Closures

```lox
fun makeAdder(n) {
    return fun(x) { return x + n; };
}
var addFive = makeAdder(5);
print addFive(3);  // 8
print addFive(10); // 15
```

### Lists and Higher-Order Functions

```lox
fun map(list, f) {
    var result = [];
    var i = 0;
    while (i < len(list)) {
        push(result, f(list[i]));
        i = i + 1;
    }
    return result;
}

var nums = [1, 2, 3, 4, 5];
var doubled = map(nums, fun(x) { return x * 2; });
print doubled; // [2, 4, 6, 8, 10]
```

### Classes and Inheritance

```lox
class Shape {
    init(color) { this.color = color; }
    describe { return this.color + " " + this.name; }
    class create(color) { return Shape(color); }
}

class Circle < Shape {
    init(color, radius) {
        super.init(color);
        this.radius = radius;
    }
    name { return "circle"; }
    area { return 3.14159 * this.radius * this.radius; }
}

var c = Circle("red", 5);
print c.describe; // red circle
print c.area;     // 78.53...
```

### Ternary

```lox
fun printShape(shape) {
    var kind = shape.area > 50 ? "large" : "small";
    print shape.describe + " is " + kind;
}
```

### Recursion

```lox
fun fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
print fib(10); // 55
```

## Language Reference

For full grammar rules, native functions, and differences from the book see [LANGUAGE.md](LANGUAGE.md).
