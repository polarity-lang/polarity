Line comments are written using two dashes: `-- This is a comment`.
Certain items of the program can also be annotated with a documentation comment.
Here is an example using doc-comments:

```xfn
-- | The type of booleans
data Bool {
  -- | The boolean truth value
  True,
  -- | The boolean false value
  False,
}
```

These documentation comments are preserved during defunctionalization and refunctionalization.
