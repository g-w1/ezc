# Standard Library

There are some functions that are provided by the `libstd.a` standard library.

Here they are.

- `PutString(string)` Takes an array and prints all the ascii characters in it to stdout.
- `PutStringLine(string)` Same as `PutString` but prints `\n` after.
- `PutNewLine()` Prints `\n` to stdout.
- `PutChar(char)` Takes a character and prints it if it is ascii.
- `PutNum(num)` Takes a number and prints the *value*.
- `PutNumHex(num)` Takes a number and prints it in hex.
- `PutNumBin(num)` Takes a number and prints it in binary.
- `InputLine()` Reads a line from stdin and returns an array of it.
