<a href="https://polarity-lang.github.io/">
    <p align="center">
        <img alt="The polarity logo" src="https://raw.githubusercontent.com/polarity-lang/artwork/88e3b8f9e4c87a0baf6a0a61f0a7e5e9f1d757a2/logo_transparent.svg" width=30%>
    </p>
</a>

<h1 align="center">Polarity</h1>
<p align="center"><strong>A programming language with dependent data and codata types.</strong></p>

<p align="center">
    <a href="https://polarity-lang.github.io/">
        <img src="https://img.shields.io/website-up-down-green-red/http/polarity-lang.github.io" alt="Website">
    </a>
    <a href="https://github.com/polarity-lang/polarity/actions/workflows/ci.yml">
        <img src="https://github.com/polarity-lang/polarity/actions/workflows/ci.yml/badge.svg" alt="Rust CI">
    </a>
    <a href="https://app.codecov.io/gh/polarity-lang/polarity">
        <img src="https://codecov.io/gh/polarity-lang/polarity/branch/main/graph/badge.svg" alt="Codecov Coverage">
    </a>
</p>

<br>

## Community

Feel welcome to join our [Discord server](https://discord.gg/NWjGr9qNhR).

## Requirements

Install Rust (e.g. via [rustup.rs](https://rustup.rs/)).

## Installation

Installation instructions and the language documentation is available at [polarity-lang.github.io](https://polarity-lang.github.io/).

To locally install the executable, run:

```sh
make install
```

By default, it gets installed to `~/.cargo/bin/pol`.

## Quickstart

From the root of this repository, run:

```sh
pol run examples/example.pol
```

To pretty-print a file, run:

```sh
pol fmt examples/example.pol
```

For more information about the CLI, run:

```sh
pol --help
```

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── lang                    Language implementation
│   ├── ast                 Definition of the abstract syntax tree (untyped and typed)
│   ├── docs                Generate HTML documentation
│   ├── driver              Demand-driven compiler driver used by the binary, LSP and test-runner
│   ├── elaborator          Elaborating an untyped syntax tree into a typed syntax tree.
│   ├── lowering            Lowering concrete to (untyped) abstract syntax tree
│   ├── lsp                 LSP language server implementation
│   ├── miette_util         Convert source code spans
│   ├── parser              Concrete syntax tree (cst), lexer and parser
│   ├── printer             Print abstract syntax tree to text
│   └── transformations     Source-to-Source transformations available as code actions.
│                           (E.g. lifting and de- and refunctionalization.)
├── std                     The Polarity Standard Library
├── test                    Integration tests
│   ├── suites              Test cases
│   └── test-runner         Test runner
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.

## Tracing Support

The compiler uses the [log crate](https://crates.io/crates/log) to trace useful diagnostic information during its execution.
The emitting of the logs is controlled via environment variables and the [env-logger crate](https://crates.io/crates/env_logger).
The site for that crate contains a lot of information about all available options.
The two flags `--trace` and `--debug` can also be used to configure the output.

A simple invocation which writes trace information to the console is:

```sh
RUST_LOG=trace pol run examples/example.pol
```

The testsuite uses the same logging infrastructure as the main application, so any options used for the `pol` binary should also work for the `test-runner` binary.


## Licenses

This project is distributed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
