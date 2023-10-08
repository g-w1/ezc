# Installing ez

## The compiler

Installing the compiler for `ez`, `ezc` is fairly simple.

First, make sure you have `cargo` and `zig` in your `$PATH` and are on x86_64 linux. You should use zig `0.7.1`. If you are on windows, [wsl](https://docs.microsoft.com/en-us/windows/wsl/install-win10) should work (although it has not been tested on wsl).

Then run `cargo install ezc`. This will download and compile `ezc` for you. To test if it worked run `ezc` in the terminal. If it says:


```
ERROR: I need an input file.
```
you have installed it correctly.

> Note if you want to install without zig, run `HAS_NO_ZIG=1 cargo install ezc`.

## Standard Library

To build the standard library, run `git clone https://github.com/g-w1/ezc && cd ezc/lib && zig build`. If this works correctly, you should have a copy of the standard library in `ezc/lib/zig-cache/lib/libstd.a`.

## Testing if it worked

To test if everything is installed correctly have a file called `hello_world.ez` with this contents:
```
External function PutStringLine(s).
Set hello_world to "Hello World!".
Set tmp to PutStringLine(hello_world).
```

To compile it run `ezc hello_world.ez -stdlib-path path/to/stdlib.a`.

To see if it worked, the command `./a.out` should print `"Hello World!"`

## Troubleshooting

If you have any issues with the build or install process please raise an [issue on github](https://github.com/g-w1/ezc/issues).
