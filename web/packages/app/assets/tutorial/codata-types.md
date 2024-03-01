Codata types are specified by a list of methods or destructors.
A very simple example is the type of pairs of a boolean and a natural number:

```pol
data Bool { True, False }
data Nat { Z, S(n: Nat)}
codata Pair {
  proj1: Bool,
  proj2: Nat
}
```

This type supports two observations; the first observations `proj1` yields a boolean value when invoked on a `Pair`, and the observation `proj2` yields a natural number.

A common codata type that is typically built into many programming languages is the function type.
In our language, it is not built-in, but we can define it as follows:

```pol
codata Fun(a b: Type) {
    Fun(a, b).ap(a: Type, b: Type, x: a) : b
}
```

Codata types can also model infinite types. The type of infinite streams is a classical example and written like this:

```pol
codata Stream(a: Type) {
  Stream(a).head(a: Type) : a,
  Stream(a).tail(a: Type) : Stream(a),
}
```

Sometimes we also need to reference the object on which a method is invoked in its return type.
This is especially the case when we want an observation to yield a proof that the object satisfies some property.
Here is a simple example which shows how this can be expressed:

```pol
codata Bool {
  Bool.neg : Bool,
  (x: Bool).neg_inverse : Eq(Bool, x, x.neg.neg),
}
```
