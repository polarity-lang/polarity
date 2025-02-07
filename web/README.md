# Web Demo

Based on [tower-lsp-web-demo](https://github.com/silvanshade/tower-lsp-web-demo/), commit `d629bf80cab03e8f87dcd5821e1307c204ca6a9e`.

## Requirements

* [Rust and Cargo](https://www.rust-lang.org/tools/install)
* [Node.js](https://nodejs.org/en/download) and [npm](https://www.npmjs.com/package/npm)

## Build

```sh
make deps
make build
```

## Run

To run the web demo, execute the following command:

```sh
make run
```

Then, navigate to [http://localhost:9000/editor#example.pol](http://localhost:9000/editor#example.pol), where `example.pol` can be any file in the `examples` directory.

## Troubleshooting

If you experience the following error:

```
FATAL ERROR: Reached heap limit Allocation failed - JavaScript heap out of memory
```

consider increasing the heap size, e.g. by setting the `NODE_OPTIONS` environment variable:

```sh
export NODE_OPTIONS=--max-old-space-size=4096
```

## License

The content in this folder is based on [tower-lsp-web-demo](https://github.com/silvanshade/tower-lsp-web-demo/) by Darin Morrison.
Like the rest of the project, it is licensed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](../LICENSE-APACHE) and [LICENSE-MIT](../LICENSE-MIT) for details.
