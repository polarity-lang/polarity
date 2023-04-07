The simplest form of data types do not have parameters or indices.
In that case, the constructors of the data type can be given as a comma-separated list.
As with all syntactic constructs, we always allow trailing commas.

```xfn
data Bool { True, False, }
```

In the more general case we have to specify the precise type that a constructor constructs.
Therefore, the above data type declaration can be written more explicitly as:

```xfn
data Bool { True: Bool, False: Bool }
```

A simple example of a parameterized type is the type of singly-linked lists of some type `a`.
In that case, we have to specify both the parameters of the type constructor `List`, and the instantiations of the term constructors `Nil` and `Cons`.
For the parameter of the type constructor `List` we make use of the impredicative type universe, which is written `Type`.

```xfn
data List(a: Type) {
  Nil(a: Type): List(a),
  Cons(a: Type, x: a, xs: List(a)): List(a)
}
```

A proper dependent type is the type of length-indexed lists: the vector type.
The `VNil` and `VCons` constructors of vectors create vectors with different indices.

```xfn
data Nat { Z, S(n: Nat) }
data Vec(a: Type, n: Nat) {
  VNil(a: Type): Vec(a, Z),
  VCons(a: Type, n: Nat, x: a, xs: Vec(a, n)): Vec(a, S(n))
}
```

Finally, we can define the Martin-Löf equality type as follows:

```xfn
data Eq (a: Type, x y: a) {
    Refl(a: Type, x: a) : Eq(a, x, x)
}
```
