An incomplete program can be written using typed holes.
Typed holes are written using either `?` or `...`; they have type `?` which unifies with any other type.
For example, an incomplete implementation of boolean negation can be written as follows:

```xfn
def Bool.neg : Bool {
  True => ?,
  False => ?,
}
```
