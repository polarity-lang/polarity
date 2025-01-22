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
    type Ctx = LevelCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.binders
    }
}

impl Ctx {
    pub(super) fn disambiguate_name(&self, name: VarBind) -> VarBind {
        let (id, span) = match name {
            VarBind::Var { span, id } => (id, span),
            VarBind::Wildcard { span } => ("x".to_string(), span),
        };

        let mut name = VarBind::Var { span, id };
        while self.contains_name(&name) {
            name = increment_name(name);
        }
        name
    }

    fn contains_name(&self, name: &VarBind) -> bool {
        for telescope in &self.binders.bound {
            if telescope.iter().any(|binder| &binder.name == name) {
                return true;
            }
        }
        false
    }
}
