# Loops

Loops are a core part of any programming language.

Here is an example of loops in ez:
```
set index to 0.
loop,
  if index >= 10,
    break.
  !
  change index to index + 1.
!
```

You can use the break keyword to get out of loop.

You can also nest loops. You *cannot* set variables in loops because they would be shadowed on the next iteration of the loop. Variable shadowing is *not* allowed in ez.
