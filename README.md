# Xfunc Implementation

Extending De-/Refunctionalization to Dependent Types

## Requirements

Install Rust (e.g. via [rustup.rs](https://rustup.rs/)).

## Installation

To locally install the executable, run:

```sh
make install
```

By default, it gets installed to `~/.cargo/bin/xfunc`.

Optionally, if you have `npm` and VSCode installed, this will also build and install the VSCode extension.

## Quickstart

From the root of this repository, run:

```sh
xfunc run examples/example.xfn
```

Enable verbose output by supplying the `--trace` option like so:

```sh
xfunc --trace run examples/example.xfn
```

To pretty-print a file, run:

```sh
xfunc fmt examples/example.xfn
```

For more information about the CLI, run:

```sh
xfunc --help
```

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── ext/vscode              VSCode extension
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
% Color definitions for XFN
\definecolor{xfnBlack}{rgb}{0,0,0}
\definecolor{xfnBlue}{rgb}{0.06, 0.2, 0.65}
\definecolor{xfnGreen}{RGB}{0,155,85}
\definecolor{xfnRed}{rgb}{0.8,0.4,0.3}
\definecolor{xfnCyan}{rgb}{0.0, 1.0, 1.0}
\definecolor{xfnMagenta}{rgb}{0.8, 0.13, 0.13}
\definecolor{xfnYellow}{rgb}{0.91, 0.84, 0.42}
\definecolor{xfnWhite}{rgb}{1,1,1}
```
