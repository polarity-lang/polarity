After all other data types, codata types, definitions and codefinitions an additional expression can be written.
This is called the "main" expression of the program.

```xfn
data Bool { True, False }
def Bool.neg {
  True => False,
  False => True,
}
True.neg
```
