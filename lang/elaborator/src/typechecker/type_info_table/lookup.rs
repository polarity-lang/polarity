use ast::*;

use super::{
    CodefMeta, CtorMeta, DefMeta, DtorMeta, LetMeta, TyCtorMeta, TypeError, TypeInfoTable,
};

impl TypeInfoTable {
    pub fn lookup_ctor_or_codef(&self, name: &Ident) -> Result<CtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_ctor.get(name) {
                return Ok(meta.clone());
            }
            if let Some(meta) = map.map_codef.get(name) {
                return Ok(meta.to_ctor().clone());
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level ctor or codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor_or_def(&self, name: &Ident) -> Result<DtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_dtor.get(name) {
                return Ok(meta.clone());
            }
            if let Some(meta) = map.map_def.get(name) {
                return Ok(meta.to_dtor().clone());
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level dtor or def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_let(&self, name: &Ident) -> Result<&LetMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_let.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level let {name} not found"),
            span: None,
        })
    }

    pub fn lookup_tyctor(&self, name: &Ident) -> Result<&TyCtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_tyctor.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level tyctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_codef(&self, name: &Ident) -> Result<&CodefMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_codef.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_ctor(&self, name: &Ident) -> Result<&CtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_ctor.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level ctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_def(&self, name: &Ident) -> Result<&DefMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_def.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor(&self, name: &Ident) -> Result<&DtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_dtor.get(name) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level dtor {name} not found"),
            span: None,
        })
    }
}
