use crate::Backend;
use crate::ir::Module;
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

pub fn rename_ir(ir: &mut Module, backend: Backend) {
    let mut ctx = RenameCtx::new(backend);
    ir.rename(&mut ctx);
}

pub fn rename_ir_for_js(ir: &mut Module) {
    rename_ir(ir, Backend::Javascript);
}

#[derive(Debug, Clone)]
pub struct RenameCtx {
    pub bindings: Vec<Binding>,
    pub backend: Backend,
}

impl RenameCtx {
    pub fn new(backend: Backend) -> Self {
        Self { bindings: Vec::new(), backend }
    }

    pub fn rename_binder(&mut self, ident: &mut Ident) {
        let original = ident.clone();
        self.rename_to_valid_identifier(&mut ident.name);
        self.disambiguate_ident(ident);
        self.bindings.push(Binding { original, renamed: ident.clone() });
    }

    pub fn rename_binders(&mut self, idents: &mut [Ident]) {
        idents.iter_mut().for_each(|ident| self.rename_binder(ident));
    }

    fn disambiguate_ident(&self, ident: &mut Ident) {
        let occupied_ids: Vec<_> = self
            .bindings
            .iter()
            .filter(|other| *ident.name == other.renamed.name)
            .map(|other| other.renamed.id)
            .collect();

        if occupied_ids.contains(&ident.id) {
            // find smallest non-occupied id
            if !occupied_ids.contains(&None) {
                ident.id = None;
            } else {
                for id in 0.. {
                    if !occupied_ids.contains(&Some(id)) {
                        ident.id = Some(id);
                        break;
                    }
                }
            }
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn rename_bound(&self, ident: &mut Ident) -> Result<(), ()> {
        let binding = self.bindings.iter().rfind(|binding| *ident == binding.original).ok_or(())?;
        *ident = binding.renamed.clone();
        Ok(())
    }

    pub fn rename_to_valid_identifier(&self, ident: &mut String) {
        match self.backend {
            Backend::Javascript => rename_to_valid_js_identifier(ident),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub original: Ident,
    pub renamed: Ident,
}

//
// Javascript
//

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

    // discard trailing digits (to avoid id conflicts)
    if ident.ends_with(|c: char| c.is_ascii_digit()) {
        *ident = ident.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
    }

    // make sure the ident is non-empty
    if ident.is_empty() {
        *ident = String::from("x");
    }

    // make sure the ident is not a JS keyword
    if JS_RESERVED_WORDS.contains(&ident.as_str()) {
        ident.push('_');
    }
}
