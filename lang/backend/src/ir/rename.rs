use miette::Diagnostic;
use thiserror::Error;

use crate::Backend;
use crate::ir::Module;
use crate::ir::ident::Ident;
use crate::result::BackendResult;

pub type RenameResult<T = ()> = Result<T, RenameError>;

pub trait Rename {
    fn rename(&mut self, ctx: &mut RenameCtx) -> RenameResult;
}

impl<T: Rename> Rename for Vec<T> {
    fn rename(&mut self, ctx: &mut RenameCtx) -> RenameResult {
        for x in self.iter_mut() {
            x.rename(ctx)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum RenameError {
    #[error("The name `{name}` is not bound in the renaming context.")]
    UnboundName { name: Ident },

    #[error("Active bindings are empty when they shouldn't.")]
    #[diagnostic(help = "This is a bug in the renaming logic.")]
    EmptyBindings,
}

pub fn rename_ir(ir: &mut Module, ctx: &mut RenameCtx) -> BackendResult {
    ir.rename(ctx).map_err(Into::into)
}

#[derive(Debug, Clone)]
pub struct RenameCtx {
    /// Bindings that are currently in scope where later entries shadow earlier ones.
    active_bindings: Vec<Binding>,
    /// Remember bindings that are not in scope anymore to ensure uniqueness of names.
    inactive_bindings: Vec<Binding>,
    pub backend: Backend,
}

impl RenameCtx {
    pub fn new(backend: Backend) -> Self {
        Self { active_bindings: Vec::new(), inactive_bindings: Vec::new(), backend }
    }

    pub fn rename_global_binder(&mut self, ident: &mut Ident) {
        let original = ident.clone();
        self.rename_to_valid_identifier(&mut ident.name);
        self.disambiguate_ident(ident);
        self.active_bindings.push(Binding { original, renamed: ident.clone() });
    }

    pub fn rename_global_binders(&mut self, idents: &mut [Ident]) {
        for ident in idents {
            self.rename_global_binder(ident);
        }
    }

    pub fn rename_local_binder<F>(&mut self, ident: &mut Ident, f: F) -> RenameResult
    where
        F: FnOnce(&mut RenameCtx) -> RenameResult,
    {
        let original = ident.clone();
        self.rename_to_valid_identifier(&mut ident.name);
        self.disambiguate_ident(ident);
        self.active_bindings.push(Binding { original, renamed: ident.clone() });
        f(self)?;
        let binding = self.active_bindings.pop().ok_or(RenameError::EmptyBindings)?;
        self.inactive_bindings.push(binding);

        Ok(())
    }

    pub fn rename_local_binders<F>(&mut self, idents: &mut [Ident], f: F) -> RenameResult
    where
        F: FnOnce(&mut RenameCtx) -> RenameResult,
    {
        match idents.split_first_mut() {
            None => f(self),
            Some((x, xs)) => {
                self.rename_local_binder(x, |extended| extended.rename_local_binders(xs, f))
            }
        }
    }

    fn disambiguate_ident(&self, ident: &mut Ident) {
        let occupied_ids: Vec<_> = self
            .active_bindings
            .iter()
            .chain(self.inactive_bindings.iter())
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

    pub fn rename_bound(&self, ident: &mut Ident) -> RenameResult {
        let binding = self
            .active_bindings
            .iter()
            .rfind(|binding| *ident == binding.original)
            .ok_or_else(|| RenameError::UnboundName { name: ident.clone() })?;
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

/// Reserved identifiers in ECMAScript/Node.
const JS_RESERVED_WORDS: &[&str] = &[
    // ECMAScript keywords
    // See <https://tc39.es/ecma262/#prod-ReservedWord>.
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
    // The following identifiers are prohibited in strict mode, which TypeScript compiles to by default.
    // The specification defines these in two separate places:
    // * https://tc39.es/ecma262/multipage/strict-mode-of-ecmascript.html#sec-strict-mode-of-ecmascript
    // * https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-identifiers-static-semantics-early-errors
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "arguments",
    "eval",
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
