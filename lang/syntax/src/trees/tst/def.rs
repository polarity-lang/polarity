//! AST with type information

use crate::generic;

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
