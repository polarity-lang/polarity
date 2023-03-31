# normalizer

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
│   ├── env.rs              
│   ├── eval.rs             Reflecting an expression as a value
│   ├── lib.rs              List of modules
│   ├── normalize.rs        The composition of eval and readback
│   ├── read_back.rs        Reifying a value as a normal form
│   ├── result.rs           Error messages generated during normalization
│   └── val.rs              Elements of the semantic domain
```