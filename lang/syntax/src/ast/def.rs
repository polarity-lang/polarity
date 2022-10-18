use codespan::Span;

use crate::common::Ident;
use crate::generic;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AST;

impl generic::Phase for AST {
    type Info = Info;
    type TypeInfo = Info;

    type VarName = Ident;
}

pub type Prg = generic::Prg<AST>;
pub type Decls = generic::Decls<AST>;
pub type Decl = generic::Decl<AST>;
pub type Data = generic::Data<AST>;
pub type Codata = generic::Codata<AST>;
pub type Impl = generic::Impl<AST>;
pub type TypAbs = generic::TypAbs<AST>;
pub type Ctor = generic::Ctor<AST>;
pub type Dtor = generic::Dtor<AST>;
pub type Def = generic::Def<AST>;
pub type Codef = generic::Codef<AST>;
pub type Match = generic::Match<AST>;
pub type Comatch = generic::Comatch<AST>;
pub type Case = generic::Case<AST>;
pub type Cocase = generic::Cocase<AST>;
pub type TypApp = generic::TypApp<AST>;
pub type Exp = generic::Exp<AST>;
pub type Telescope = generic::Telescope<AST>;
pub type Params = generic::Params<AST>;
pub type Args = generic::Args<AST>;
pub type Param = generic::Param<AST>;

#[derive(Debug, Clone)]
pub struct Info {
    pub span: Option<Span>,
}

impl Info {
    pub fn empty() -> Self {
        Self { span: None }
    }
}
