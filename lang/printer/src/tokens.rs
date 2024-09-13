//! This module contains the symbols and keywords of the surface language.
//! These constants are used when we prettyprint source code.

// Symbols
//
//

/// The symbol `->`
pub const ARROW: &str = "->";

/// The symbol `=>`
pub const FAT_ARROW: &str = "=>";

/// The symbol `,`
pub const COMMA: &str = ",";

/// The symbol `:`
pub const COLON: &str = ":";

/// The symbol `.`
pub const DOT: &str = ".";

/// The symbol `@`
pub const AT: &str = "@";

/// The symbol `?`
pub const HOLE: &str = "?";

/// The symbol `#`
pub const HASH: &str = "#";
pub const COLONEQ: &str = ":=";

// Keywords
//
//

/// The keyword `data`
pub const DATA: &str = "data";

/// The keyword `codata`
pub const CODATA: &str = "codata";

/// The keyword `def`
pub const DEF: &str = "def";

/// The keyword `codef`
pub const CODEF: &str = "codef";

/// The keyword `let`
pub const LET: &str = "let";

/// The keyword `match`
pub const MATCH: &str = "match";

/// The keyword `as`
pub const AS: &str = "as";

/// The keyword `comatch`
pub const COMATCH: &str = "comatch";

/// The keyword `absurd`
pub const ABSURD: &str = "absurd";

/// The keyword `Type`
pub const TYPE: &str = "Type";

/// The keyword `implicit`
pub const IMPLICIT: &str = "implicit";
