Codefinitions create producers (or objects) for codata types.
They need to define the behavior of the object when each of its destructors is invoked.
Analogously to pattern matching, where we pattern match on the constructors of a data type, we *copattern match* on the destructors of a codata type.
For example, we can create a pair with the `Pair` codata type defined above as follows:

```xfn
codef MyPair: Pair {
    proj1 => True,
    proj2 => 42,
}
```

We can retrieve the values in the pair by invoking one of the destructors.
For instance, `MyPair.proj2` will yield the result of `42`.

Codefinitions can also be used to construct infinite objects.
For instance, we can generate an infinite stream that counts upwards as follows:

```xfn
codef CountUp(n: Nat): Stream(Nat) {
    head(_) => n,
    tail(_) => CountUp(S(n)),
}
```

Finally, codefinitions can also return proofs that they fulfill certain properties:

```xfn
codef True: Bool {
    not => False,
    neg_inverse => Refl(Bool, True),
}

codef False: Bool {
    not => True,
    neg_inverse => Refl(Bool, False),
}
```
