use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::print_comma_separated;
use printer::theme::ThemeExt;
use printer::tokens::CODATA;
use printer::tokens::CODEF;
use printer::tokens::COLON;
use printer::tokens::COMMA;
use printer::tokens::DATA;
use printer::tokens::DEF;
use printer::tokens::DOT;
use printer::tokens::HASH;
use printer::tokens::IMPLICIT;
use printer::tokens::LET;
use printer::tokens::USE;
use printer::util::BracesExt;
use printer::util::IsNilExt;
use printer::Alloc;
use printer::Builder;
use printer::Print;
use printer::PrintCfg;
use url::Url;

use crate::ctx::LevelCtx;
use crate::named::Named;
use crate::shift_and_clone;
use crate::Zonk;

use super::exp::*;
use super::ident::*;
use super::traits::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::HashMap;

fn print_return_type<'a, T: Print>(
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
    ret_typ: &'a T,
) -> Builder<'a> {
    alloc
        .line_()
        .append(COLON)
        .append(alloc.space())
        .append(ret_typ.print(cfg, alloc).group())
        .nest(cfg.indent)
}

#[derive(Debug, Clone)]
pub struct DocComment {
    pub docs: Vec<String>,
}

impl Print for DocComment {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let DocComment { docs } = self;
        let prefix = "-- | ";
        alloc.concat(
            docs.iter().map(|doc| {
                alloc.comment(prefix).append(alloc.comment(doc)).append(alloc.hardline())
            }),
        )
    }
}

/// A single attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    /// Declarations with this annotation are omitted during prettyprinting.
    OmitPrint,
    /// A transparent let-binding is expanded during normalization.
    Transparent,
    /// An opaque let-binding is not expanded during normalization.
    Opaque,
    /// The compiler does not know about the meaning of this annotation.
    Other(String),
}

impl Print for Attribute {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Attribute::OmitPrint => alloc.text("omit_print"),
            Attribute::Opaque => alloc.text("opaque"),
            Attribute::Transparent => alloc.text("transparent"),
            Attribute::Other(s) => alloc.text(s),
        }
    }
}

/// An attribute can be attached to various nodes in the syntax tree.
/// We use the same syntax for attributes as Rust, that is `#[attr1,attr2]`.
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub attrs: Vec<Attribute>,
}

impl Print for Attributes {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.attrs.is_empty() {
            alloc.nil()
        } else {
            let p = print_comma_separated(&self.attrs, cfg, alloc);
            alloc.text(HASH).append(p.brackets()).append(alloc.hardline())
        }
    }
}

impl Attributes {
    /// Checks whether the `#[omit_print]` attribute is present.
    fn is_visible(&self) -> bool {
        !self.attrs.contains(&Attribute::OmitPrint)
    }
}

// Module
//
//

/// The state of a metavariable during the elaboration phase.
/// All metavariables start in the unsolved state, but as we
/// learn more information during elaboration we find out what
/// precise terms the metavariables stand for.
///
/// A metavariable is always annotated with a local context which specifies
/// which free variables may occur in the solution.
#[derive(Debug, Clone)]
pub enum MetaVarState {
    /// We know what the metavariable stands for.
    Solved { ctx: LevelCtx, solution: Box<Exp> },
    /// We don't know yet what the metavariable stands for.
    Unsolved { ctx: LevelCtx },
}

impl MetaVarState {
    /// Returns the found solution for the metavariable, if it exists.
    pub fn solution(&self) -> Option<Box<Exp>> {
        match self {
            MetaVarState::Solved { solution, .. } => Some(solution.clone()),
            MetaVarState::Unsolved { .. } => None,
        }
    }

    /// Returns true if the metavariable is solved.
    pub fn is_solved(&self) -> bool {
        match self {
            MetaVarState::Solved { .. } => true,
            MetaVarState::Unsolved { .. } => false,
        }
    }
}

/// A use declaration
///
/// ```text
/// use "Data/Bool.pol"
/// ```
#[derive(Debug, Clone)]
pub struct UseDecl {
    pub span: Span,
    pub path: String,
}

impl Print for UseDecl {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let UseDecl { path, .. } = self;
        alloc.text(USE).append(alloc.space()).append(alloc.text(path).double_quotes())
    }
}

/// A module containing declarations
///
/// There is a 1-1 correspondence between modules and files in our system.
#[derive(Debug, Clone)]
pub struct Module {
    /// The location of the module on disk
    pub uri: Url,
    /// List of module imports at the top of a module.
    pub use_decls: Vec<UseDecl>,
    /// Declarations contained in the module other than imports.
    pub decls: Vec<Decl>,
    /// Metavariables that were generated for this module during lowering.
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
}

impl Module {
    pub fn xdefs_for_type(&self, type_name: &str) -> Vec<Ident> {
        let mut out = vec![];

        for decl in &self.decls {
            match decl {
                Decl::Def(def) => {
                    if def.self_param.typ.name.id == type_name {
                        out.push(def.name.clone());
                    }
                }
                Decl::Codef(codef) => {
                    if codef.typ.name.id == type_name {
                        out.push(codef.name.clone());
                    }
                }
                _ => {}
            }
        }

        out
    }

    pub fn xtors_for_type(&self, type_name: &str) -> Vec<Ident> {
        let mut out = vec![];

        for decl in &self.decls {
            match decl {
                Decl::Data(data) => {
                    if data.name.id == type_name {
                        for ctor in &data.ctors {
                            out.push(ctor.name.clone());
                        }
                    }
                }
                Decl::Codata(codata) => {
                    if codata.name.id == type_name {
                        for dtor in &codata.dtors {
                            out.push(dtor.name.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        out
    }

    pub fn lookup_decl(&self, name: &Ident) -> Option<&Decl> {
        self.decls.iter().find(|decl| decl.name() == name)
    }

    pub fn lookup_let(&self, name: &Ident) -> Option<&Let> {
        self.decls.iter().find_map(|decl| match decl {
            Decl::Let(tl_let) if tl_let.name == *name => Some(tl_let),
            _ => None,
        })
    }

    pub fn lookup_def(&self, name: &Ident) -> Option<&Def> {
        self.decls.iter().find_map(|decl| match decl {
            Decl::Def(def) if def.name == *name => Some(def),
            _ => None,
        })
    }

    pub fn lookup_codef(&self, name: &Ident) -> Option<&Codef> {
        self.decls.iter().find_map(|decl| match decl {
            Decl::Codef(codef) if codef.name == *name => Some(codef),
            _ => None,
        })
    }

    pub fn find_main(&self) -> Option<Box<Exp>> {
        self.decls.iter().find_map(|decl| match decl {
            Decl::Let(tl_let) if tl_let.is_main() => Some(tl_let.body.clone()),
            _ => None,
        })
    }
}

impl Print for Module {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Module { use_decls, decls, .. } = self;

        // UseDecls
        //
        //

        let use_decls =
            alloc.intersperse(use_decls.iter().map(|decl| decl.print(cfg, alloc)), alloc.line());

        // Decls
        //
        //

        // We usually separate declarations with an empty line, except when the `omit_decl_sep` option is set.
        // This is useful for typesetting examples in papers which have to make economic use of vertical space.
        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };

        let decls = decls
            .iter()
            .filter(|decl| decl.attributes().is_visible())
            .map(|decl| decl.print(cfg, alloc));

        // UseDecls + Decls
        //
        //

        if use_decls.is_nil() {
            alloc.intersperse(decls, sep)
        } else {
            use_decls
                .append(alloc.line())
                .append(alloc.line())
                .append(alloc.intersperse(decls, sep))
        }
    }
}

// Decl
//
//

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Def(Def),
    Codef(Codef),
    Let(Let),
}

impl From<Data> for Decl {
    fn from(data: Data) -> Self {
        Decl::Data(data)
    }
}

impl From<Codata> for Decl {
    fn from(codata: Codata) -> Self {
        Decl::Codata(codata)
    }
}

impl From<Def> for Decl {
    fn from(def: Def) -> Self {
        Decl::Def(def)
    }
}

impl From<Codef> for Decl {
    fn from(codef: Codef) -> Self {
        Decl::Codef(codef)
    }
}

impl From<Let> for Decl {
    fn from(tl_let: Let) -> Self {
        Decl::Let(tl_let)
    }
}

impl Decl {
    pub fn attributes(&self) -> &Attributes {
        match self {
            Decl::Data(Data { attr, .. }) => attr,
            Decl::Codata(Codata { attr, .. }) => attr,
            Decl::Def(Def { attr, .. }) => attr,
            Decl::Codef(Codef { attr, .. }) => attr,
            Decl::Let(Let { attr, .. }) => attr,
        }
    }

    /// Returns whether the declaration is the "main" expression of the module.
    pub fn get_main(&self) -> Option<Let> {
        match self {
            Decl::Let(tl_let) => tl_let.is_main().then(|| tl_let.clone()),
            _ => None,
        }
    }
}

impl Named for Decl {
    fn name(&self) -> &Ident {
        match self {
            Decl::Data(Data { name, .. }) => name,
            Decl::Codata(Codata { name, .. }) => name,
            Decl::Def(Def { name, .. }) => name,
            Decl::Codef(Codef { name, .. }) => name,
            Decl::Let(Let { name, .. }) => name,
        }
    }
}

impl HasSpan for Decl {
    fn span(&self) -> Option<Span> {
        match self {
            Decl::Data(data) => data.span,
            Decl::Codata(codata) => codata.span,
            Decl::Def(def) => def.span,
            Decl::Codef(codef) => codef.span,
            Decl::Let(tl_let) => tl_let.span,
        }
    }
}

impl Print for Decl {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Decl::Data(data) => data.print(cfg, alloc),
            Decl::Codata(codata) => codata.print(cfg, alloc),
            Decl::Def(def) => def.print(cfg, alloc),
            Decl::Codef(codef) => codef.print(cfg, alloc),
            Decl::Let(tl_let) => tl_let.print(cfg, alloc),
        }
    }
}

impl Zonk for Decl {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        match self {
            Decl::Data(data) => data.zonk(meta_vars),
            Decl::Codata(codata) => codata.zonk(meta_vars),
            Decl::Def(def) => def.zonk(meta_vars),
            Decl::Codef(codef) => codef.zonk(meta_vars),
            Decl::Let(tl_let) => tl_let.zonk(meta_vars),
        }
    }
}

// Data
//
//

#[derive(Debug, Clone)]
pub struct Data {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attributes,
    pub typ: Box<Telescope>,
    pub ctors: Vec<Ctor>,
}

impl Print for Data {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Data { span: _, doc, name, attr, typ, ctors } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(attr.print(cfg, alloc))
            .append(alloc.keyword(DATA))
            .append(alloc.space())
            .append(alloc.typ(&name.id))
            .append(typ.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if ctors.is_empty() {
            empty_braces(alloc)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(ctors.iter().map(|ctor| ctor.print(cfg, alloc)), sep))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_anno()
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
    }
}

impl Zonk for Data {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Data { span: _, doc: _, name: _, attr: _, typ, ctors } = self;
        typ.zonk(meta_vars)?;
        for ctor in ctors {
            ctor.zonk(meta_vars)?;
        }
        Ok(())
    }
}

// Codata
//
//

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attributes,
    pub typ: Box<Telescope>,
    pub dtors: Vec<Dtor>,
}

impl Print for Codata {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codata { span: _, doc, name, attr, typ, dtors } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(attr.print(cfg, alloc))
            .append(alloc.keyword(CODATA))
            .append(alloc.space())
            .append(alloc.typ(&name.id))
            .append(typ.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if dtors.is_empty() {
            empty_braces(alloc)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(dtors.iter().map(|dtor| dtor.print(cfg, alloc)), sep))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_anno()
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
    }
}

impl Zonk for Codata {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Codata { span: _, doc: _, name: _, attr: _, typ, dtors } = self;
        typ.zonk(meta_vars)?;
        for dtor in dtors {
            dtor.zonk(meta_vars)?;
        }
        Ok(())
    }
}

// Ctor
//
//

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypCtor,
}

impl Print for Ctor {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Ctor { span: _, doc, name, params, typ } = self;

        let doc = doc.print(cfg, alloc);
        let head = alloc.ctor(&name.id).append(params.print(cfg, alloc));

        let head = if typ.is_simple() {
            head
        } else {
            let mut cfg = cfg.clone();
            cfg.print_function_sugar = false;
            head.append(print_return_type(&cfg, alloc, typ)).group()
        };
        doc.append(head)
    }
}

impl Zonk for Ctor {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Ctor { span: _, doc: _, name: _, params, typ } = self;
        params.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        Ok(())
    }
}

// Dtor
//
//

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Box<Exp>,
}

impl Print for Dtor {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { span: _, doc, name, params, self_param, ret_typ } = self;

        let doc = doc.print(cfg, alloc);
        let head = if self_param.is_simple() {
            alloc.text(DOT)
        } else {
            self_param.print(&PrintCfg { print_function_sugar: false, ..*cfg }, alloc).append(DOT)
        };
        let head = head
            .append(alloc.dtor(&name.id))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group();
        doc.append(head)
    }
}

impl Zonk for Dtor {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Dtor { span: _, doc: _, name: _, params, self_param, ret_typ } = self;
        params.zonk(meta_vars)?;
        self_param.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        Ok(())
    }
}

// Def
//
//

#[derive(Debug, Clone)]
pub struct Def {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attributes,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Box<Exp>,
    pub cases: Vec<Case>,
}

impl Def {
    pub fn to_dtor(&self) -> Dtor {
        Dtor {
            span: self.span,
            doc: self.doc.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            self_param: self.self_param.clone(),
            ret_typ: self.ret_typ.clone(),
        }
    }
}

impl Print for Def {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { span: _, doc, name, attr, params, self_param, ret_typ, cases } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let doc = doc.print(cfg, alloc).append(attr.print(cfg, alloc));

        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(self_param.print(cfg, alloc))
            .append(DOT)
            .append(alloc.dtor(&name.id))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group();

        let body = print_cases(cases, cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl Zonk for Def {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Def { span: _, doc: _, name: _, attr: _, params, self_param, ret_typ, cases } = self;
        params.zonk(meta_vars)?;
        self_param.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

// Codef
//
//

#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attributes,
    pub params: Telescope,
    pub typ: TypCtor,
    pub cases: Vec<Case>,
}

impl Codef {
    pub fn to_ctor(&self) -> Ctor {
        Ctor {
            span: self.span,
            doc: self.doc.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            typ: self.typ.clone(),
        }
    }
}

impl Print for Codef {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { span: _, doc, name, attr, params, typ, cases } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let doc = doc.print(cfg, alloc).append(attr.print(cfg, alloc));

        let head = alloc
            .keyword(CODEF)
            .append(alloc.space())
            .append(alloc.ctor(&name.id))
            .append(params.print(cfg, alloc))
            .append(print_return_type(
                &PrintCfg { print_function_sugar: false, ..*cfg },
                alloc,
                typ,
            ))
            .group();

        let body = print_cases(cases, cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl Zonk for Codef {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Codef { span: _, doc: _, name: _, attr: _, params, typ, cases } = self;
        params.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

// Let
//
//

#[derive(Debug, Clone)]
pub struct Let {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attributes,
    pub params: Telescope,
    pub typ: Box<Exp>,
    pub body: Box<Exp>,
}

impl Let {
    /// Returns whether the declaration is the "main" expression of the module.
    pub fn is_main(&self) -> bool {
        self.name.id == "main" && self.params.is_empty()
    }
}

impl Print for Let {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Let { span: _, doc, name, attr, params, typ, body } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let doc = doc.print(cfg, alloc).append(attr.print(cfg, alloc));

        let head = alloc
            .keyword(LET)
            .append(alloc.space())
            .append(&name.id)
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, typ))
            .group();

        let body = body.print(cfg, alloc).braces_anno();

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl Zonk for Let {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let Let { span: _, doc: _, name: _, attr: _, params, typ, body } = self;
        params.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        body.zonk(meta_vars)?;
        Ok(())
    }
}

// SelfParam
//
//

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub info: Option<Span>,
    pub name: Option<Ident>,
    pub typ: TypCtor,
}

impl SelfParam {
    pub fn telescope(&self) -> Telescope {
        Telescope {
            params: vec![Param {
                implicit: false,
                name: self.name.clone().unwrap_or_else(|| Ident::from_string("")),
                typ: Box::new(self.typ.to_exp()),
            }],
        }
    }

    /// A self parameter is simple if the list of arguments to the type is empty, and the name is None.
    /// If the self parameter is simple, we can omit it during prettyprinting.
    pub fn is_simple(&self) -> bool {
        self.typ.is_simple() && self.name.is_none()
    }
}

impl Print for SelfParam {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let SelfParam { info: _, name, typ } = self;

        let mut cfg = cfg.clone();
        cfg.print_function_sugar = false;

        match name {
            Some(name) => alloc
                .text(&name.id)
                .append(COLON)
                .append(alloc.space())
                .append(typ.print(&cfg, alloc))
                .parens(),
            None => typ.print(&cfg, alloc),
        }
    }
}

impl Zonk for SelfParam {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        let SelfParam { info: _, name: _, typ } = self;
        typ.zonk(meta_vars)?;
        Ok(())
    }
}

// Telescope
//
//

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Telescope {
    pub params: Vec<Param>,
}

impl Telescope {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn instantiate(&self) -> TelescopeInst {
        let params = self
            .params
            .iter()
            .map(|Param { name, .. }| ParamInst {
                span: None,
                name: name.clone(),
                info: None,
                typ: None,
            })
            .collect();
        TelescopeInst { params }
    }
}

impl Print for Telescope {
    /// This function tries to "chunk" successive blocks of parameters which have the same type.
    /// For example, instead of printing `x: Type, y: Type` we print `x y: Type`. We do this by
    /// remembering in the `running` variable what the current type of the parameters is, and
    /// whether we can append the current parameter to this list. There are two complications:
    ///
    /// 1) Due to de Bruijn indices we have to shift the types when we compare them. For example,
    ///    instead of printing `n: Nat, x: Vec(Bool,n), y: Vec(Bool,n)` we want to print
    ///    `n: Nat, x y: Vec(Bool,n)`. But in de Bruijn notation this list looks like
    ///    `_: Nat, _ : Vec(0), _: Vec(1)`.
    ///
    /// 2) We cannot chunk two parameters if one is implicit and the other isn't, even if they have
    ///    the same type. For example: `implicit a: Type, b: Type` cannot be chunked.
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Telescope { params } = self;
        let mut output = alloc.nil();
        if params.is_empty() {
            return output;
        };
        // Running stands for the type and implicitness of the current "chunk" we are building.
        let mut running: Option<(&Exp, bool)> = None;
        for Param { implicit, name, typ } in params {
            match running {
                // We need to shift before comparing to ensure we compare the correct De-Bruijn indices
                Some((rtype, rimplicit))
                    if shift_and_clone(rtype, (0, 1)) == **typ && rimplicit == *implicit =>
                {
                    // We are adding another parameter of the same type.
                    output = output.append(alloc.space()).append(alloc.text(&name.id));
                }
                Some((rtype, _)) => {
                    // We are adding another parameter with a different type,
                    // and have to close the previous list first.
                    output = output
                        .append(COLON)
                        .append(alloc.space())
                        .append(rtype.print(cfg, alloc))
                        .append(COMMA)
                        .append(alloc.line());
                    if *implicit {
                        output = output
                            .append(IMPLICIT)
                            .append(alloc.space())
                            .append(alloc.text(&name.id));
                    } else {
                        output = output.append(alloc.text(&name.id));
                    }
                }
                None => {
                    // We are starting a new chunk and adding the very first parameter.
                    // If we are starting a chunk of implicit parameters then we also have to
                    // add the "implicit" keyword at this point.
                    if *implicit {
                        output = output.append(IMPLICIT).append(alloc.space())
                    }

                    output = output.append(alloc.text(&name.id));
                }
            }
            running = Some((typ, *implicit));
        }
        // Close the last parameter
        match running {
            None => {}
            Some((rtype, _)) => {
                output = output.append(COLON).append(alloc.space()).append(rtype.print(cfg, alloc));
            }
        }
        output.append(alloc.line_()).align().parens().group()
    }
}

impl Zonk for Telescope {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        for param in &mut self.params {
            param.zonk(meta_vars)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod print_telescope_tests {

    use super::*;

    #[test]
    fn print_empty() {
        let tele = Telescope { params: vec![] };
        assert_eq!(tele.print_to_string(Default::default()), "")
    }

    #[test]
    fn print_simple_chunk() {
        let param1 = Param {
            implicit: false,
            name: Ident::from_string("x"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let param2 = Param {
            implicit: false,
            name: Ident::from_string("y"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(x y: Type)")
    }

    #[test]
    fn print_simple_implicit_chunk() {
        let param1 = Param {
            implicit: true,
            name: Ident::from_string("x"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let param2 = Param {
            implicit: true,
            name: Ident::from_string("y"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(implicit x y: Type)")
    }

    #[test]
    fn print_mixed_implicit_chunk_1() {
        let param1 = Param {
            implicit: true,
            name: Ident::from_string("x"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let param2 = Param {
            implicit: false,
            name: Ident::from_string("y"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(implicit x: Type, y: Type)")
    }

    #[test]
    fn print_mixed_implicit_chunk_2() {
        let param1 = Param {
            implicit: false,
            name: Ident::from_string("x"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let param2 = Param {
            implicit: true,
            name: Ident::from_string("y"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(x: Type, implicit y: Type)")
    }

    #[test]
    fn print_shifting_example() {
        let param1 = Param {
            implicit: false,
            name: Ident::from_string("a"),
            typ: Box::new(TypeUniv::new().into()),
        };
        let param2 = Param {
            implicit: false,
            name: Ident::from_string("x"),
            typ: Box::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: 0 },
                name: Ident::from_string("a"),
                inferred_type: None,
            })),
        };
        let param3 = Param {
            implicit: false,
            name: Ident::from_string("y"),
            typ: Box::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: 1 },
                name: Ident::from_string("a"),
                inferred_type: None,
            })),
        };
        let tele = Telescope { params: vec![param1, param2, param3] };
        assert_eq!(tele.print_to_string(Default::default()), "(a: Type, x y: a)")
    }
}

// Param
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Param {
    pub implicit: bool,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    pub typ: Box<Exp>,
}

impl Named for Param {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Substitutable for Param {
    type Result = Param;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Param { implicit, name, typ } = self;
        Param { implicit: *implicit, name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl Print for Param {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { implicit, name, typ } = self;
        if *implicit {
            alloc
                .text(IMPLICIT)
                .append(alloc.space())
                .append(&name.id)
                .append(COLON)
                .append(alloc.space())
                .append(typ.print(cfg, alloc))
        } else {
            alloc.text(&name.id).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
        }
    }
}

impl Zonk for Param {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), crate::ZonkError> {
        self.typ.zonk(meta_vars)
    }
}
