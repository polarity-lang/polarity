//! # Renaming
//!
//! The AST representation primarily uses nameless DeBruijn indizes and levels to track
//! the binding structure between binding and bound occurrences of variables.
//! If we want to print a human-readable representation of the AST that can be
//! parsed again, then we have to invent new names for the nameless variables which
//! reflect the same binding structure: The creation of these new names is called "renaming".
//!
//! ## Example
//!
//! The nameless representation of the const function which returns the first of two arguments
//! is `\_ => \_ => 1`; renaming makes up variable names `x` and `y` to obtain the renamed term
//! `\x => \y => x`.
//!
//! ## Implementation
//!
//! We traverse the AST while maintaining a context of variable names that are bound.
//! Every time we come across a binding occurrence we check whether the name that is currently
//! annotated is already bound in the context. If it isn't bound then we leave it unchanged,
//! otherwise we choose a new name which is not already bound in the context.
//! Every time we encounter a variable we look up the name in the context.

use crate::{
    VarBind,
    ctx::{BindContext, GenericCtx, LevelCtx, values::Binder},
};

pub trait Rename: Sized {
    /// Assigns consistent names to all binding and bound variable occurrences.
    /// Should only be called on closed expressions or declarations.
    fn rename(&mut self) {
        let mut ctx = GenericCtx::empty().into();
        self.rename_in_ctx(&mut ctx)
    }
    /// Assigns consistent names to all binding and bound variable occurrences.
    /// The provided `ctx` must contain names for all free variables of `self`.
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx);
}

impl<R: Rename + Clone> Rename for Box<R> {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        (**self).rename_in_ctx(ctx);
    }
}

impl<T: Rename> Rename for Option<T> {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        match self {
            None => (),
            Some(x) => x.rename_in_ctx(ctx),
        }
    }
}

impl<T: Rename> Rename for Vec<T> {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.iter_mut().for_each(|x| x.rename_in_ctx(ctx))
    }
}

impl Rename for () {
    fn rename_in_ctx(&mut self, _ctx: &mut RenameCtx) {}
}

impl<T: Rename> Rename for Binder<T> {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        // In expressions, `Binder` is currently only used to track the names of hole arguments.
        // These do not need to be renamed because they don't show up in the printed output.
        // They are solely used for better error messages.
        let Binder { name: _, content } = self;

        content.rename_in_ctx(ctx);
    }
}

pub struct RenameCtx {
    pub binders: LevelCtx,
}

impl From<GenericCtx<()>> for RenameCtx {
    fn from(binders: GenericCtx<()>) -> Self {
        RenameCtx { binders }
    }
}

impl BindContext for RenameCtx {
    type Content = ();

    fn ctx_mut(&mut self) -> &mut LevelCtx {
        &mut self.binders
    }
}

impl RenameCtx {
    pub fn disambiguate_var_bind(&self, var: VarBind) -> VarBind {
        let (mut name, span) = match var {
            VarBind::Var { span, id } => (id, span),
            VarBind::Wildcard { span } => ("x".to_string(), span),
        };

        while self.contains_name(&name) {
            name = increment_name(name);
        }

        VarBind::Var { span, id: name }
    }

    fn contains_name(&self, name: &str) -> bool {
        for telescope in &self.binders.bound {
            if telescope.iter().any(|binder| match &binder.name {
                VarBind::Var { id, .. } => id == name,
                VarBind::Wildcard { .. } => false,
            }) {
                return true;
            }
        }
        false
    }
}

pub fn increment_name(mut name: String) -> String {
    if name.ends_with('\'') {
        name.push('\'');
        return name;
    }
    let (s, digits) = split_trailing_digits(&name);
    match digits {
        None => format!("{s}0"),
        Some(n) => format!("{s}{}", n + 1),
    }
}

pub fn split_trailing_digits(s: &str) -> (&str, Option<usize>) {
    let n_digits = s.chars().rev().take_while(char::is_ascii_digit).count();
    let (s, digits) = s.split_at(s.len() - n_digits);

    (s, digits.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_empty() {
        assert_eq!(split_trailing_digits(""), ("", None))
    }

    #[test]
    fn test_split_no_digits() {
        assert_eq!(split_trailing_digits("foo"), ("foo", None))
    }

    #[test]
    fn test_split_digits() {
        assert_eq!(split_trailing_digits("foo42"), ("foo", Some(42)))
    }
}
