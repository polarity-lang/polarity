use crate::Backend;
use crate::ir::ident::Ident;

pub trait Rename {
    fn rename(&mut self, ctx: &mut RenameCtx);
}

impl<T: Rename> Rename for Vec<T> {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        for x in self.iter_mut() {
            x.rename(ctx);
        }
    }
}

pub struct RenameCtx {
    pub binders: Vec<(String, Ident)>,
    pub backend: Backend,
}

pub fn rename_to_valid_identifer(ident: &mut String, backend: Backend) {
    match backend {
        Backend::Javascript => rename_to_valid_js_identifier(ident),
    }
}

/// Reserved words in ECMAScript.
///
/// See <https://tc39.es/ecma262/#prod-ReservedWord>.
const JS_RESERVED_WORDS: [&str; 38] = [
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

fn rename_to_valid_js_identifier(ident: &mut String) {
    // discard unicode and '
    *ident = ident.chars().filter(|&c| c.is_ascii() && c != '\'').collect();

    // make sure the ident is non-empty
    if ident == "" {
        *ident = String::from("x");
    }

    // make sure the ident is not a JS keyword
    if JS_RESERVED_WORDS.contains(&ident.as_str()) {
        ident.push('_');
    }
}
