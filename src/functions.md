# Functions

Functions are also core to any programming language. If you are not familiar with functions they are like mini-programs that can take arguments and return results.

Here is an example of one in ez:
```
Function AddOne(number),
  set result to number + 1.
  Return result.
!
```

We can then call a function with an argument to get the return value:

```
Set six to AddOne(5). {
  This calls AddOne with the `number` argument as 5.
  Then the function is run with the number variable set to 5.
  Finally, it returns 6.
}
```

In ez, comments start with `{` and end with `}`. Anything inside the comment is ignored.

A `Return` statement immediately stops the function and returns what is after it.

Recursive functions **are** allowed:
```
Function fib(a),
    if a <= 2,
        return 1.
    !
    return fib(a - 1) + fib(a - 2).
!

set result to fib(10).
```

In this example, at the end, result is set to 55, which is the 10th Fibonacci number. Notice how the fib function can call itself.
