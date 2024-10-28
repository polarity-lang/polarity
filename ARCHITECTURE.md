# The Architecture of the Polarity Compiler

Following the reasoning laid out by [matklad](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html), we give an overview of the architecture of the polarity compiler.
This documentation is intended for users who want to contribute patches or features, not for users who just want to use the language.

## Implementation of the Core Logic

The core logic of the compiler, that is, all the relevant compiler intermediate stages, are implemented in the `lang/` subdirectory.

### Lexical Analysis

The syntax of polarity is not whitespace sensitive and is analyzed using a generated lexer and parser.
All code related to lexing and parsing, as well as the datatypes used to represent a parsed syntax tree, live in the `lang/parser` subdirectory as part of the `parser` crate.
The lexer is generated with the help of the [logos](https://docs.rs/logos/latest/logos/) crate, and the specification of tokens and the lexer lives in `lang/parser/src/lexer/mod.rs`.
The parser is generated using the [lalrpop](https://lalrpop.github.io/lalrpop/) parser generator, and specified in `lang/parser/src/grammar/cst.lalrpop`.
The generated parser annotates (almost) all syntax nodes with information about the source span, since this information is needed for error messages and the LSP implementation.

We use a separate syntax representation for the result of parsing that we call the *concrete syntax tree (CST)*.
The CST, which is defined in the files in  `lang/parser/src/cst/..`, should mirror the productions of the parser as closely as possible.
We do not parse with a symbol table, so when we encounter `f` in the token stream, for example, we cannot tell yet whether `f` is a locally bound variable or a function defined at the toplevel.
The CST therefore does not have separate constructors for variables and function calls, these must be distinguished in the lowering phase which comes after parsing.

### Lowering

Lowering is the process of taking the *concrete syntax tree* that is the result from parsing and turning it into an *abstract syntax tree* (defined in `lang/ast/`) that is used in all later stages of the compiler.
Here is what happens during lowering:

- The binding structure is recorded using De Bruijn indices instead of a named representation. We do not, however, forget the original names that were written by the programmer, because these names are important when we generate error messages.
- While we cannot distinguish whether `F(e1,e2,e3)` refers to a constructor, a codefinition or a let-bound definition in the concrete syntax tree, we do distinguish these three different cases in the abstract syntax tree and annotate the syntax nodes accordingly.
- We record whether a symbol is defined in the local module, or whether it is imported from another module.
- We throw errors if a symbol is declared multiple times in the same module, or if the number of arguments to a a call is not consistent with the arity of that call.
- We generate fresh unsolved metavariables for every occurrence of a typed hole `_` or `?`, and for every implicit argument that is not passed explicitly.

In order to make the lowering transformation possible we first have to generate symbol tables for the module we are lowering and for all imported modules.
The symbol table for a module contains information about all names that are defined at the toplevel of that module.
These names include type constructors, constructors and destructors introduced in data and codata declarations, as well as toplevel definitions, codefinitions and let-bound definitions.
With each such name we record its arity: The arity consists in the number of required arguments, as well as the presence of implicit arguments.
All the code which concerns symbol tables is contained in the `lang/lowering/src/symbol_table` subdirectory.
The driver is responsible for computing the symbol tables of the module and all imported modules and to make them available during lowering.


### Elaboration

Elaboration takes an abstract syntax tree that is the result of lowering, typechecks it, and annotates the syntax tree with typing information.
The elaboration algorithm consists of three components that depend on each other:

- A normalizer, defined in `lang/elaborator/src/normalizer`, takes an abstract syntax tree and brings it into normal form using an untyped normalization by evaluation algorithm.
- A unifier, defined in `lang/elaborator/src/unifier`, takes equality constraints and checks whether the constraint is solvable.
  If the constraint is solvable then it returns a solution.
- The typechecker, defined in `lang/elaborator/src/typechecker` contains the core implementation of a bidirectional typechecker for all the constructs available in the language.

Typechecking requires information about the type signatures of toplevel definitions.
This information is recorded in `type_info_tables`, which are very similar to the `symbol_tables` that we used during lowering, but which contain more information about the types of arguments.
Again, it is the responsibility of the driver to first compute the type info tables for the module and all imported modules before starting to typecheck the contents of the module.

### The Compiler Driver

The compiler driver, defined in `lang/driver`, contains the logic for orchestrating the interplay between parsing, lowering and typechecking, and is also responsible for making code actions available that can then be served by the LSP server to clients.
Because we implement such a language server which gives live feedback to the programmer we cannot use the architecture of a batch compiler for our implementation.
Instead, we use the architecture of a demand-driven build tool, where we have "build recipes" for building different artifacts (such as the lowered ast, typechecked ast, symbol table, type info table, etc) for each module. These recipes can then invoke other recipes to build the artifacts that they depend on.
For example, the recipe for lowering a module will invoke the recipe for computing the symbol table.
Artifacts are then cached inside an in-memory database and only invalidated if the file itself or one of its dependencies changes.

### Various utilities

Some functionalities are implemented in their own crates in the `lang/` directory.
We have chosen this architecture to keep the design more modular.

- The `printer` crate in `lang/printer` instantiates a Wadler-Leijen style pretty-printing library and defines multiple backends.
  These backends allow to write to a file, write colored output on the console, and generate latex to be used in documentation and articles.
- The `transformations` crate defines source-to-source transformations that are made available through the CLI and the language server. In particular, it contains the implementation of defunctionalization and refunctionalization.
- The `lsp` crate contains the implementation of the language server.

## Users of the Core Logic

There are three binaries which use the logic defined in the `lang/` subdirectory.

- The `pol` CLI which allows to use the language on the command-line is defined in the `app` directory. That directory contains all the logic for parsing command-line arguments and executing the desired actions. This binary should not contain complicated logic involving compilation; this logic should be implemented in `lang/driver` instead.
- The testsuite runner defined in the `test/test-runner` directory. This testsuite runner directly instantiates a driver session and uses this session to test various stages of the compiler separately.
- The webdemo defined in the `web` directory which is served on [polarity-lang.github.io](polarity-lang.github.io). The webdemo is built by compiling the core logic to web assembly using the Rust web assembly backend. The webdemo then runs entirely in the browser of the client.