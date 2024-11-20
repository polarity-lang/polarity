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

## Quickstart

Before installing anything on your machine you can try out polarity in the browser on [polarity-lang.github.io](https://polarity-lang.github.io/). The website also contains complete installation instructions, language documentation and a guide on how to configure editor support using our language server.
If you want to install polarity locally on your system you can follow these steps:

- Install a Rust toolchain using [rustup.rs](https://rustup.rs/).
- Clone the repository on your machine:
  ```console
  git clone git@github.com:polarity-lang/polarity.git
  ```
- To locally install the executable, run:
  ```console
  make install
  ```
  The binary `pol` gets installed to `~/.cargo/bin/pol`; make sure that this directory is in your `$PATH`.
- From the root of this repository, run:
  ```console
  $ pol run examples/example.pol 
  S(S(S(S(S(Z)))))
  ```
- For more information about available subcommands, run:
  ```console
  pol --help
  ```

## Contributing

Pull requests, bug reports and feature requests are highly welcomed and encouraged!
If you want to contribute yourself, understand the code, or run the testsuite, then you can find more developer-focused documentation in the [CONTRIBUTING.md](CONTRIBUTING.md).

## Licenses

This project is distributed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
