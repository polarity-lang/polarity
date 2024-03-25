# Elaboration

This crate contains the code of the elaborator who takes a program  which hasn't been typechecked and produces a fully elaborated program where all types of every subexpression are fully annotated.

The elaborator consists of three main parts:

- The `normalizer` computes the normal form of expressions which is needed to check if two terms are convertible.
- The `unifier` solves a set of equality constraints and produces a most general unifier.
- The `typechecker` traverses the untyped syntax tree, normalizes subexpressions, generates unification problems and outputs a fully elaborated syntax tree.

## Normalizer

Implementation of untyped normalization-by-evaluation.

```text
  +---------+           +-------+                +----+
  |         |  --eval-> |  Val  | --read_back--> |    |
  |  Exp    |           +-------+                | Nf |
  |         |                                    |    |
  |         | <------------forget--------------- |    |
  +---------+                                    +----+
```

```text
├── src 
|   ├── normalizer
│   |   ├── env.rs              
│   |   ├── eval.rs             Reflecting an expression as a value
│   |   ├── lib.rs              List of modules
│   |   ├── normalize.rs        The composition of eval and readback
│   |   ├── read_back.rs        Reifying a value as a normal form
│   |   ├── result.rs           Error messages generated during normalization
│   |   └── val.rs              Elements of the semantic domain
|   ...
```

