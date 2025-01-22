use printer::Print;

use ast::ctx::values::{Binder as TypeCtxBinder, TypeCtx};

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

impl From<TypeCtxBinder<Box<ast::Exp>>> for Binder {
    fn from(binder: TypeCtxBinder<Box<ast::Exp>>) -> Self {
        match binder.name {
            ast::VarBind::Var { id, .. } => {
                Binder::Var { name: id, typ: binder.content.print_to_string(None) }
            }
            ast::VarBind::Wildcard { .. } => {
                Binder::Wildcard { typ: binder.content.print_to_string(None) }
            }
        }
    }
}
