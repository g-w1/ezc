# Modules

Ez has a very minimal module system.

It is similar to c, except no preprocessor or `.h` files.

Functions are the only thing that can be exported from `.ez` files to other files.

Here is an example:

`lib.ez`
```
Export function MulByTwo(n),
  return n * n.
!
```

`main.ez`
```
External function MulByTwo(n).

set twelve to MulByTwo(6).
```

To compile this into a single binary, run:
```bash
# this produces a lib.ez.o file
$ ezc lib.ez -lib
# this produces a main.ez.o file
$ ezc main.ez -nolink
# this links everything together
$ ld main.ez.o lib.ez.o -o out
```

For another example of this, with a Makefile, go here: [https://github.com/g-w1/ezc/tree/master/tests/module](https://github.com/g-w1/ezc/tree/master/tests/module).

Ez uses the [systemv x86_64](https://wiki.osdev.org/System_V_ABI) so you can also import and export functions to c or any language that can use this abi in the exact same way.

For an example of interfacing with c go here: [https://github.com/g-w1/ezc/tree/master/tests/cinterface](https://github.com/g-w1/ezc/tree/master/tests/cinterface)
