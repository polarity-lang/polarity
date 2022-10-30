use codespan::Span;

use crate::common::HasSpan;
use crate::common::Ident;
use crate::de_bruijn::*;

use super::generic;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UST;

impl generic::Phase for UST {
    type Info = Info;
    type TypeInfo = Info;

    type VarName = Ident;

    fn print_var(name: &Self::VarName, _idx: Idx) -> String {
        name.clone()
    }
}

pub type Prg = generic::Prg<UST>;
pub type Decls = generic::Decls<UST>;
pub type Decl = generic::Decl<UST>;
pub type Data = generic::Data<UST>;
pub type Codata = generic::Codata<UST>;
pub type Impl = generic::Impl<UST>;
pub type TypAbs = generic::TypAbs<UST>;
pub type Ctor = generic::Ctor<UST>;
pub type Dtor = generic::Dtor<UST>;
pub type Def = generic::Def<UST>;
pub type Codef = generic::Codef<UST>;
pub type Match = generic::Match<UST>;
pub type Comatch = generic::Comatch<UST>;
pub type Case = generic::Case<UST>;
pub type Cocase = generic::Cocase<UST>;
pub type TypApp = generic::TypApp<UST>;
pub type Exp = generic::Exp<UST>;
pub type Telescope = generic::Telescope<UST>;
pub type TelescopeInst = generic::TelescopeInst<UST>;
pub type Params = generic::Params<UST>;
pub type Args = generic::Args<UST>;
pub type Param = generic::Param<UST>;
pub type ParamInst = generic::ParamInst<UST>;

#[derive(Default, Debug, Clone)]
pub struct Info {
    pub span: Option<Span>,
}

impl Info {
    pub fn empty() -> Self {
        Self { span: None }
    }
}

impl HasSpan for Info {
    fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }
}
