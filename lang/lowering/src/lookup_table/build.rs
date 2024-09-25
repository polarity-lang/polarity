use ast::HashMap;
use decls::*;
use miette_util::ToMiette;
use parser::cst::*;

use crate::LoweringError;

use super::{DeclMeta, LookupTable};

pub fn build_lookup_table(module: &Module) -> Result<LookupTable, LoweringError> {
    let mut lookup_table = LookupTable { map: HashMap::default() };

    let Module { decls, .. } = module;

    for decl in decls {
        decl.build(&mut lookup_table)?;
    }

    Ok(lookup_table)
}

trait BuildLookupTable {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError>;
}

impl BuildLookupTable for Decl {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        match self {
            Decl::Data(data) => data.build(lookup_table),
            Decl::Codata(codata) => codata.build(lookup_table),
            Decl::Def(def) => def.build(lookup_table),
            Decl::Codef(codef) => codef.build(lookup_table),
            Decl::Let(tl_let) => tl_let.build(lookup_table),
        }
    }
}

impl BuildLookupTable for Data {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Data { span, name, params, ctors, .. } = self;
        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Data { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        for ctor in ctors {
            ctor.build(lookup_table)?;
        }
        Ok(())
    }
}

impl BuildLookupTable for Ctor {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Ctor { span, name, params, .. } = self;
        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Ctor { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildLookupTable for Codata {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Codata { span, name, params, dtors, .. } = self;
        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Codata { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        for dtor in dtors {
            dtor.build(lookup_table)?;
        }
        Ok(())
    }
}

impl BuildLookupTable for Dtor {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Dtor { span, name, params, .. } = self;
        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Dtor { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildLookupTable for Def {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Def { span, name, params, .. } = self;

        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Def { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildLookupTable for Codef {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Codef { span, name, params, .. } = self;

        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Codef { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildLookupTable for Let {
    fn build(&self, lookup_table: &mut LookupTable) -> Result<(), LoweringError> {
        let Let { span, name, params, .. } = self;
        match lookup_table.map.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Let { params: params.clone() };
                lookup_table.map.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}
