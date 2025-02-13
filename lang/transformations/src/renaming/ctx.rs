use ast::ctx::GenericCtx;
use ast::*;
use ctx::{BindContext, LevelCtx};

use super::util::increment_name;

pub struct Ctx {
    pub binders: LevelCtx,
}

impl From<GenericCtx<()>> for Ctx {
    fn from(binders: GenericCtx<()>) -> Self {
        Ctx { binders }
    }
}

impl BindContext for Ctx {
    type Content = ();

    fn ctx_mut(&mut self) -> &mut LevelCtx {
        &mut self.binders
    }
}

impl Ctx {
    pub(super) fn disambiguate_var_bind(&self, var: VarBind) -> VarBind {
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
