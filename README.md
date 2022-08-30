# Xfunc Implementation

Extending De-/Refunctionalization to the Lambda Cube

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── ext/vscode              VSCode extension
├── lang                    Language implementation
│   ├── core                Core (typechecker, evaluator)
│   ├── lowering            Lowering concrete to abstract syntax tree
│   ├── parser              Parse text to concrete syntax tree
│   ├── printer             Print abstract syntax tree to text
│   └── syntax              Syntax tree definitions
├── lsp                     LSP language server implementation
├── scripts                 Utility scripts
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.
