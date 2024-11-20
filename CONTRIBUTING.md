# Developer and Contributer Information

You can find a high-level overview of the architecture of the compiler in [ARCHITECTURE.md](ARCHITECTURE.md).
This document contains information about the developer workflow, including our git conventions, information about how to use the testsuite effectively, and a guide to using the debugging infrastructure.

## Contents

- [Project Structure](#project-structure)
- [Testsuite](#testsuite)
- [Code Coverage](#code-coverage)
- [Debugging](#debugging)
- [Web Demo](#web-demo)
- [Linters and Formatters](#linters-and-formatters)
- [Pull Requests](#pull-requests)
- [Releases](#releases)

## Project structure

```text
├── app                     CLI application
├── examples                Example code in the object language
├── lang                    Language implementation
│   ├── ast                 Definition of the abstract syntax tree (untyped and typed)
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

## Testsuite

We have two kinds of tests: unit tests and integration tests.
Unit tests use the [default testing infrastructure](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) that is provided by Rust and Cargo.
Integration tests use polarity source files as input and perform various checks on them.
They are executed using the custom test runner that is implemented in the `test/test-runner` directory.


> [!TIP]
> You can run *all* unit and integration tests using the `make test` target of the Makefile, or manually using `cargo test --workspace`.


## Code Coverage

We do monitor the code coverage provided by our testsuite in order to diagnose which parts of the codebas are not tested sufficiently, but we do not implement coverage thresholds that a pull request needs to fulfil in order to be merged.
You can inspect the current coverage of the main branch through the web interface of codecov using [this link](https://app.codecov.io/gh/polarity-lang/polarity).

> [!TIP]
> You can also compute code coverage locally. To do this, first install the llvm-cov subcommand for cargo using ` cargo install cargo-llvm-cov`. You can then run `make coverage` to get a html report of the parts of the code that are covered by tests.

## Debugging

The compiler uses the [log crate](https://crates.io/crates/log) to trace useful diagnostic information during its execution.
Which logs are emitted is controlled through environment variables and the [env-logger crate](https://crates.io/crates/env_logger).
The documentation of that crate provides documentation for all available options.
The two flags `--trace` and `--debug` can also be used to configure the output.


> [!TIP]
> A simple invocation which writes debug information to the console is:
>
> ```sh
> RUST_LOG=trace pol run examples/example.pol
> ```


The testsuite uses the same logging infrastructure as the main application, so any options used for the `pol` binary should also work for the `test-runner` binary.

## Web Demo

xxx

## Linters and Formatters

Every pull request must adhere to the code formatting standard implemented by `cargo fmt`, and must not produce warnings when linted with `cargo clippy`.

> [!TIP]
> You can run the formatter and all linters using `make lint`, or use the git pre-commit hook provided in `scripts/git-hooks/pre-commit`.

## Pull Requests

All changes to the codebase should go through pull-requests.
We do not allow commits directly on the `main` branch of the repository.
Furthermore, all pull requests should be associated with at least one specific issue unless the PR fixes a minor problem.
Please check that you observe the following guidelines:

- If your PR changes the observable behaviour of the binary, then you have to add an entry to the `CHANGELOG.md` file with your PR under the `Unreleased` section of the changelog.
- Every PR needs to have at least 1 approval before it can be merged.
- We enforce a linear history on the `main` branch, so every PR must either be rebased or rebased and squashed before it is merged into `main`.

## Releases

> [!CAUTION]
> We do not provide versioned releases yet, so this section can be ignored for now.

We use the following workflow for generating a new release for version `x.x.x`:

- Open a branch with the name `prepare-release-x.x.x` and create a corresponding PR.
- Change the versions in all the `Cargo.toml` files to the new version `x.x.x`, build the project, and also commit the generated `Cargo.lock` file.
- Move everything under the section `Unreleased` in the `CHANGELOG.md` into a new section `[x.x.x] YYYY-MM-DD` with the current date.
- Merge the Pull request into `main`.
- In the main branch, use `git tag -a x.x.x -m "Version x.x.x` to create a tag, and `git push origin x.x.x` to publish the tag.
