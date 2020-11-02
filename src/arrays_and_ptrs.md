# Arrays and Pointers

Arrays and pointers are two of the most advanced topics in ez. They are also experimental: there are likely to be bugs when using them.

## Arrays

You can set a variable to an array in ez:

```
set array_of_nums to [1,2,3].
```

You can index an array like this:

```
set array_of_nums to [3,2,1].
set one to array_of_nums[1]. { We know one is 3. }
set array_of_nums[1] to 1. { Now array_of_nums is [1,2,1]. }
```

A string is just an array of characters, which is just an array of numbers:

```
set array_of_nums to "hello\n".
set array_of_nums[1] to 'm'. { array_of_nums is "mello\n"}
```

To find the length of an array, you can use the zeroth element of it:

```
set array_of_nums to "hello\n".
set len to array_of_nums[0]. { len is 6 }
```

> Note: for technical reasons, arrays are layed out in memory with the first element being a pointer to itself (for technical reasons involving the `mov` and `lea` instructions), the second element being the length of the array (not including the first 2 elements) and the rest being the array in memory
>
> `[1,2,3]` is this: ``pointer to itself, 3, 1, 2, 3 ``
> To see examples of interacting with this slightly different array abi, view [`lib.zig`](https://github.com/g-w1/ezc/blob/master/lib/src/lib.zig)

## Pointers

In ez, pointers are numbers that represent a pointer to a value.

Use the `@` operator to dereference a pointer:

```
set null to 0.
set oof to @null.
```

This will produce a segmentation fault; it is the classic example of dereferencing a null pointer.

> Note: when setting something to an array value, or passing an array in a function, you are just using the pointer to the array.
