# Polarity

A programming language with dependent data and codata types.
Installation instructions and the language documentation is available at [polarity-lang.github.io](https://polarity-lang.github.io/).

## Requirements

Install Rust (e.g. via [rustup.rs](https://rustup.rs/)).

## Installation

To locally install the executable, run:

```sh
make install
```

By default, it gets installed to `~/.cargo/bin/xfunc`.

## Quickstart

From the root of this repository, run:

```sh
xfunc run examples/example.pol
```

Enable verbose output by supplying the `--trace` option like so:

```sh
xfunc --trace run examples/example.pol
```

To pretty-print a file, run:

```sh
xfunc fmt examples/example.pol
```

For more information about the CLI, run:

```sh
xfunc --help
```

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── lang                    Language implementation
│   ├── lifting             Lift local (co)matches to top-level definitions
│   ├── lowering            Lowering concrete to (untyped) abstract syntax tree
│   ├── normalizer          Implementation of normalization-by-evaluation algorithm
│   ├── parser              Parse text to concrete syntax tree
│   ├── printer             Print abstract syntax tree to text
│   ├── query               Index data structures for annotated source code files and spans
│   ├── renaming            Rename abstract syntax tree s.t. it can be reparsed
│   ├── syntax              Syntax tree definitions
│   ├── typechecker         Bidirectional type inference
│   ├── unifier             Unification algorithm
│   └── xfunc               De-/Refunctionalization implementation
├── oopsla_examples         Examples from the OOPSLA paper
├── test                    Integration tests
│   ├── suites              Test cases
│   └── test-runner         Test runner
├── util                    Utility libraries
│   ├── console             Print function that works native and on WASM
│   ├── data                Collection of convenient data structures
│   ├── lsp                 LSP language server implementation
│   ├── miette_util         Convert source code spans
│   ├── tracer              Debugging library for generating trace output
│   └── tracer_macros       Procedural macros that generate trace output
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.

## Latex Output

The `xfunc` binary supports the `xfunc texify` subcommand which translates code into typeset latex snippets.
In order for color highlighting to function correctly, some colors have to be defined in the preamble. We suggest the following definition as a starting point.

```tex
\usepackage{xcolor}
% Color definitions for pol
\definecolor{polBlack}{rgb}{0,0,0}
\definecolor{polBlue}{rgb}{0.06, 0.2, 0.65}
\definecolor{polGreen}{RGB}{0,155,85}
\definecolor{polRed}{rgb}{0.8,0.4,0.3}
\definecolor{polCyan}{rgb}{0.0, 1.0, 1.0}
\definecolor{polMagenta}{rgb}{0.8, 0.13, 0.13}
\definecolor{polYellow}{rgb}{0.91, 0.84, 0.42}
\definecolor{polWhite}{rgb}{1,1,1}
```
