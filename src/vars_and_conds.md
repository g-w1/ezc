# Variables and Conditionals

## Variables

This tutorial will cover how to program in ez by using examples. Every example will showcase a different feature of ez.

The simplest ez program that does something is this:
```
Set var to 42.
```

All this program does is set a variable called var to the number 42.

If we wanted to change var to something else after we have set it we could do this:
```
Set var to 42.
Change var to var * 2.
```

This is self explanatory, we are changing var to something else. To use the `change` keyword with a variable, you must `set` it first.

> Note: keywords in `ez` that start a sentence are not case sensitive `set var to 42.` works. `to` is case sensitive because it is in the middle of a sentence. You can think of this like English.

Numbers and characters can be used interchangeably:
```
Set var to 'a'.
Change var to var * 2.
```
In this example, at the end, var is 194, because 'a' is converted to it's ASCII code. There is *0* difference between characters and numbers.

## Conditionals

We can use an `if` statement to control the flow of the code:

```
Set special_number to 0.
if special_number = 0,
  change special_number to special_number + 1.
!
```

Take a moment to think: what is the value of `special_number` at the end of this program.

The answer: 1.

The way conditionals evaluate in ez is that if the if statement guard (in this case special_number = 0) evaluates to 1 then the block is run.

Blocks look like `, (code) !`. Whitespace has *no* impact on ez.

You can create boolean expressions using these operators:
- `=` equals
- `!=` not equal
- `and` binary and operator (it also works for booleans because math is cool)
- `or` binary or operator (it also works for booleans because math is cool)
- `>` greater than
- `<` less than
- `>=` greater than or equal
- `<=` less than or equal

You can use these operators to create numbers:
- `+` add
- `-` subtract
- `*` multiply
> Note: division is not supported because it is not safe, dividing by zero is bad.
