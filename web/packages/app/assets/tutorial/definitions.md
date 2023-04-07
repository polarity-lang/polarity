Definitions create a consumer (method) for a data type.
These consumers receive an implicit input on which they pattern match.
As a simple example, we can define Boolean negation as follows:

```xfn
def Bool.neg: Bool {
    True => False,
    False => True,
}

```

Definitions can be recursive.
For instance, we can define addition on natural numbers as follows:

```xfn
def Nat.add(y: Nat) : Nat {
    Z => y,
    S(x) => S(x.add(y)),
}
```

Definitions can also deal with parametrized types.
For instance, we can define a `map` method for the data type `List` as follows:

```xfn
def List(a).map(a b: Type, f: Fun(a, b)): List(b) {
    Nil(a) => Nil(b),
    Cons(a, x, xs) => Cons(b, f.ap(a, b, x), xs.map(a, b, f)),
}
```

Finally, to illustrate dependently typed definitions, let us give the classic example of defining `append` on length-indexed lists:

```xfn
def Vec(a, n).append(a: Type, n m: Nat, ys: Vec(a, m)) : Vec(a, n.add(m)) {
    VNil(a) => ys,
    VCons(a, n', x, xs) => VCons(a, n'.add(m), x, xs.append(a, n', m, ys))
}
```

Last, but certainly not least, the return type of a definition may not only depend on its parameters but also on its (implicit) input.
To do so, we can make the input explicit by assigning it a name.
For instance, we can prove that Boolean negation is its own inverse as follows:

```xfn
def (x: Bool).neg_inverse: Eq(Bool, x, x.not.not) {
    True => Refl(Bool, True),
    False => Refl(Bool, False)
}
```
