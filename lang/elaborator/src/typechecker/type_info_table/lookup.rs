use ast::*;

use super::{CodefMeta, CtorMeta, DefMeta, DtorMeta, LetMeta, ModuleTypeInfoTable, TyCtorMeta, TypeError};

impl ModuleTypeInfoTable {
    pub fn lookup_ctor_or_codef(&self, name: &Ident) -> Result<CtorMeta, TypeError> {
        self.map_ctor
            .get(name)
            .cloned()
            .or_else(|| self.map_codef.get(name).map(|codef| codef.to_ctor()))
            .ok_or(TypeError::Impossible {
                message: format!("Top-level ctor or codef {name} not found"),
                span: None,
            })
    }

    pub fn lookup_dtor_or_def(&self, name: &Ident) -> Result<DtorMeta, TypeError> {
        self.map_dtor
            .get(name)
            .cloned()
            .or_else(|| self.map_def.get(name).map(|def| def.to_dtor()))
            .ok_or(TypeError::Impossible {
                message: format!("Top-level dtor or def {name} not found"),
                span: None,
            })
    }

    pub fn lookup_let(&self, name: &Ident) -> Result<&LetMeta, TypeError> {
        self.map_let.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level let {name} not found"),
            span: None,
        })
    }

    pub fn lookup_tyctor(&self, name: &Ident) -> Result<&TyCtorMeta, TypeError> {
        self.map_tyctor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level tyctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_codef(&self, name: &Ident) -> Result<&CodefMeta, TypeError> {
        self.map_codef.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_ctor(&self, name: &Ident) -> Result<&CtorMeta, TypeError> {
        self.map_ctor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level ctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_def(&self, name: &Ident) -> Result<&DefMeta, TypeError> {
        self.map_def.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor(&self, name: &Ident) -> Result<&DtorMeta, TypeError> {
        self.map_dtor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level dtor {name} not found"),
            span: None,
        })
    }
}
