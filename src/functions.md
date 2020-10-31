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
