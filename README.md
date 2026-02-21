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

## Example

```lox
// Fibonacci with recursion
fun fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

// Class with state and methods
class Counter {
    init(start) {
        this.value = start;
    }

    inc() {
        this.value = this.value + 1;
    }

    get() {
        return this.value;
    }
}

// Closure example
fun makeAdder(x) {
    fun add(y) {
        return x + y;
    }
    return add;
}

// Arrays + native len()
var numbers = [1, 2, 3];
print "Length:", len(numbers);

var c = Counter(10);
c.inc();
c.inc();
print "Counter value:", c.get();

var addFive = makeAdder(5);
print "5 + 10 =", addFive(10);

print "fib(6) =", fib(6);
```

## Language Reference

For full grammar rules, native functions, and differences from the book see [LANGUAGE.md](LANGUAGE.md).
