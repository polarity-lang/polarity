use ast::ctx::{Context, ContextElem, GenericCtx};
use ast::*;

use super::util::increment_name;

pub struct Ctx {
    ctx: GenericCtx<VarBind>,
}

impl From<GenericCtx<VarBind>> for Ctx {
    fn from(value: GenericCtx<VarBind>) -> Self {
        Ctx { ctx: value }
    }
}

impl Context for Ctx {
    type Elem = VarBind;

    fn push_telescope(&mut self) {
        self.ctx.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.ctx.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        //assert!(id == "_" || id.is_empty() || !self.contains_name(&elem.clone()));
        self.ctx
            .bound
            .last_mut()
            .expect("Cannot push without calling push_telescope first")
            .push(elem.clone());
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.ctx.bound.last_mut().expect(err).pop().expect(err);
    }

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.ctx.var_to_lvl(idx.into());
        self.ctx
            .bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
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
        for telescope in &self.ctx.bound {
            if telescope.contains(name) {
                return true;
            }
        }
        false
    }
}

impl ContextElem<Ctx> for Param {
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for ParamInst {
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for SelfParam {
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned().unwrap_or_else(|| VarBind::from_string(""))
    }
}
