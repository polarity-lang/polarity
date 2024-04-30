//! AST with type information

use std::rc::Rc;

use crate::generic::TypeUniv;
use crate::ust;

use crate::generic;

use super::forget::ForgetTST;

pub type Ident = generic::Ident;
pub type Label = generic::Label;
pub type DocComment = generic::DocComment;
pub type Attribute = generic::Attribute;
pub type Prg = generic::Prg;
pub type Decls = generic::Decls;
pub type Decl = generic::Decl;
pub type DataCodata<'a> = generic::DataCodata<'a>;
pub type Data = generic::Data;
pub type Codata = generic::Codata;
pub type TypAbs = generic::TypAbs;
pub type Ctor = generic::Ctor;
pub type Dtor = generic::Dtor;
pub type Def = generic::Def;
pub type Codef = generic::Codef;
pub type Let = generic::Let;
pub type Match = generic::Match;
pub type Case = generic::Case;
pub type SelfParam = generic::SelfParam;
pub type Exp = generic::Exp;
pub type Motive = generic::Motive;
pub type Telescope = generic::Telescope;
pub type TelescopeInst = generic::TelescopeInst;
pub type Args = generic::Args;
pub type Param = generic::Param;
pub type ParamInst = generic::ParamInst;

pub type TypCtor = generic::TypCtor;
pub type Call = generic::Call;
pub type DotCall = generic::DotCall;
pub type Anno = generic::Anno;
pub type LocalMatch = generic::LocalMatch;
pub type LocalComatch = generic::LocalComatch;

pub trait HasTypeInfo {
    fn typ(&self) -> Option<Rc<ust::Exp>>;
}

impl HasTypeInfo for Exp {
    fn typ(&self) -> Option<Rc<ust::Exp>> {
        match self {
            Exp::Variable(e) => e.inferred_type.clone(),
            Exp::TypCtor(_) => Some(Rc::new(ust::Exp::TypeUniv(TypeUniv { span: None }))),
            Exp::Call(e) => e.inferred_type.clone(),
            Exp::DotCall(e) => e.inferred_type.clone(),
            Exp::Anno(e) => e.normalized_type.clone(),
            Exp::TypeUniv(_) => Some(Rc::new(ust::Exp::TypeUniv(TypeUniv { span: None }))),
            Exp::LocalMatch(e) => {
                e.inferred_type.forget_tst().map(|typctor| Rc::new(ust::Exp::TypCtor(typctor)))
            }
            Exp::LocalComatch(e) => {
                e.inferred_type.forget_tst().map(|typctor| Rc::new(ust::Exp::TypCtor(typctor)))
            }
            Exp::Hole(e) => e.inferred_type.clone(),
        }
    }
}
