use ast::*;

use super::{
    CodefMeta, CtorMeta, DefMeta, DtorMeta, LetMeta, TyCtorMeta, TypeError, TypeInfoTable,
};

impl TypeInfoTable {
    pub fn lookup_data(&self, name: &IdBound) -> Result<&Data, TypeError> {
        for map in self.map.values() {
            if let Some(data) = map.map_data.get(&name.id) {
                return Ok(data);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level data type {name} not found"),
            span: None,
        })
    }

    pub fn lookup_codata(&self, name: &IdBound) -> Result<&Codata, TypeError> {
        for map in self.map.values() {
            if let Some(codata) = map.map_codata.get(&name.id) {
                return Ok(codata);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level codata type {name} not found"),
            span: None,
        })
    }

    pub fn lookup_ctor_or_codef(&self, name: &IdBound) -> Result<CtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_ctor.get(&name.id) {
                return Ok(meta.clone());
            }
            if let Some(meta) = map.map_codef.get(&name.id) {
                return Ok(meta.to_ctor().clone());
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level ctor or codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor_or_def(&self, name: &IdBound) -> Result<DtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_dtor.get(&name.id) {
                return Ok(meta.clone());
            }
            if let Some(meta) = map.map_def.get(&name.id) {
                return Ok(meta.to_dtor().clone());
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level dtor or def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_let(&self, name: &IdBound) -> Result<&LetMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_let.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level let {name} not found"),
            span: None,
        })
    }

    pub fn lookup_tyctor(&self, name: &IdBound) -> Result<&TyCtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_tyctor.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level tyctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_codef(&self, name: &IdBound) -> Result<&CodefMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_codef.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_ctor(&self, name: &IdBound) -> Result<&CtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_ctor.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level ctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_def(&self, name: &IdBound) -> Result<&DefMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_def.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor(&self, name: &IdBound) -> Result<&DtorMeta, TypeError> {
        for map in self.map.values() {
            if let Some(meta) = map.map_dtor.get(&name.id) {
                return Ok(meta);
            }
        }
        Err(TypeError::Impossible {
            message: format!("Top-level dtor {name} not found"),
            span: None,
        })
    }
}
