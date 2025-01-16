use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::{Alloc, Builder, Print, PrintCfg};

use crate::{ContainsMetaVars, Zonk, ZonkError};

use super::{Exp, MetaVar, VarBind};
// Telescope Inst
//
//

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TelescopeInst {
    pub params: Vec<ParamInst>,
}

impl TelescopeInst {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

impl Print for TelescopeInst {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
    }
}

impl Zonk for TelescopeInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TelescopeInst { params } = self;

        for param in params {
            param.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for TelescopeInst {
    fn contains_metavars(&self) -> bool {
        let TelescopeInst { params } = self;

        params.contains_metavars()
    }
}

// ParamInst
//
//

/// Instantiation of a previously declared parameter
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct ParamInst {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Box<Exp>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: VarBind,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Option<Box<Exp>>,
    /// Whether the parameter is erased during compilation.
    pub erased: bool,
}

impl Print for ParamInst {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { span: _, info: _, name, typ: _, erased: _ } = self;
        name.print(cfg, alloc)
    }
}

impl Zonk for ParamInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let ParamInst { span: _, info, name: _, typ, erased: _ } = self;

        info.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for ParamInst {
    fn contains_metavars(&self) -> bool {
        let ParamInst { span: _, info, name: _, typ, erased: _ } = self;

        info.contains_metavars() || typ.contains_metavars()
    }
}
