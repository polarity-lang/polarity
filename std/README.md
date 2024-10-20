# Standard Library

The Polarity Standard Library.

## Overview

```text
├── std                     The Polarity Standard Library
│   ├── codata              A collection of commonly used codata types
│   └── data                A collection of commonly used data types
```

## Conventions

We follow the following naming conventions in the standard library:

* Polarity filenames and folder names are in `snake_case`. Example: `data/option.pol`.
* `data` and `codata` declaration names are in `CamelCase`. Example: `data Option`.
* Destructors and top-level let bindings are in `snake_case`. Example: `Fun.ap`.
* Constructors are in `CamelCase`. Example: `Some`.

Where syntax and naming decisions are arbitrary, we loosely follow the Rust conventions.

All declarations use ASCII characters only. This is to ensure that there is always a unique natural language term to refer to any given declaration. It also ensures the standard library can easily be searched and indexed.
There may be non-ASCII Unicode characters in shorthand notation, comments and documentation strings.
