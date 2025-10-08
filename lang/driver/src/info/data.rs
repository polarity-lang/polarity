use polarity_lang_printer::Print;

use polarity_lang_ast::ctx::values::{Binder as TypeCtxBinder, Binding, TypeCtx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ctx {
    pub bound: Vec<Vec<Binder>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binder {
    Var { name: String, typ: String },
    Wildcard { typ: String },
}

impl From<TypeCtx> for Ctx {
    fn from(ctx: TypeCtx) -> Self {
        let bound =
            ctx.bound.into_iter().map(|tel| tel.into_iter().map(Into::into).collect()).collect();
        Ctx { bound }
    }
}

impl From<TypeCtxBinder<Binding>> for Binder {
    fn from(binder: TypeCtxBinder<Binding>) -> Self {
        match binder.name {
            polarity_lang_ast::VarBind::Var { id, .. } => {
                Binder::Var { name: id, typ: binder.content.typ.print_to_string(None) }
            }
            polarity_lang_ast::VarBind::Wildcard { .. } => {
                Binder::Wildcard { typ: binder.content.typ.print_to_string(None) }
            }
        }
    }
}
