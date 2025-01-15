# Elaboration

This crate contains the code of the elaborator who takes a program  which hasn't been typechecked and produces a fully elaborated program where all types of every subexpression are fully annotated.

The elaborator consists of three main parts:

- The `normalizer` computes the normal form of expressions which is needed to check if two terms are convertible.
- The algorithm in `index_unification` is used during dependent pattern matching to unify the indices of a type with those in the return type of a specific constructor or destructor.
- The algorithm in `conversion_checking` checks whether two expressions are convertible and also solves metavariables while doing so.
- The `typechecker` traverses the untyped syntax tree, normalizes subexpressions, generates unification problems and outputs a fully elaborated syntax tree.

## Normalizer

Implementation of untyped normalization-by-evaluation.

```text
  +---------+                                    +---------+
  |         |  ---------------eval-------------> |         |
  |  Exp    |                                    | Val/Neu |
  |         |                                    |         |
  |         | <------------read_back ----------- |         |
  +---------+                                    +---------+
```

```text
├── src 
|   ├── normalizer
│   |   ├── env.rs              
│   |   ├── eval.rs             Reflecting an expression as a value
│   |   ├── lib.rs              List of modules
│   |   ├── normalize.rs        The composition of eval and readback
│   |   ├── result.rs           Error messages generated during normalization
│   |   └── val.rs              Elements of the semantic domain
|   ...
```

