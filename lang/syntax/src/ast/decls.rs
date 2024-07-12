use std::rc::Rc;

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
use printer::util::BracesExt;
use printer::Alloc;
use printer::Builder;
use printer::Print;
use printer::PrintCfg;
use printer::PrintInCtx;
use url::Url;

use crate::ctx::LevelCtx;

use super::exp::*;
use super::ident::*;
use super::lookup_table::{DeclKind, LookupTable};
use super::traits::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::HashMap;
use super::Item;
use super::Shift;

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
    Solved { ctx: LevelCtx, solution: Rc<Exp> },
    /// We don't know yet what the metavariable stands for.
    Unsolved { ctx: LevelCtx },
}

impl MetaVarState {
    pub fn solution(&self) -> Option<Rc<Exp>> {
        match self {
            MetaVarState::Solved { solution, .. } => Some(solution.clone()),
            MetaVarState::Unsolved { .. } => None,
        }
    }
}
/// A module containing declarations
///
/// There is a 1-1 correspondence between modules and files in our system.
#[derive(Debug, Clone)]
pub struct Module {
    pub uri: Url,
    /// Map from identifiers to declarations
    pub map: HashMap<Ident, Decl>,
    /// Metadata on declarations
    pub lookup_table: LookupTable,
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
}

impl Module {
    pub fn find_main(&self) -> Option<Rc<Exp>> {
        let main_candidate = self.map.get("main")?.get_main()?;
        Some(main_candidate.body)
    }
}

impl Print for Module {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let items =
            self.iter().filter(|item| item.attributes().is_visible()).map(|item| match item {
                Item::Data(data) => data.print_in_ctx(cfg, self, alloc),
                Item::Codata(codata) => codata.print_in_ctx(cfg, self, alloc),
                Item::Def(def) => def.print(cfg, alloc),
                Item::Codef(codef) => codef.print(cfg, alloc),
                Item::Let(tl_let) => tl_let.print(cfg, alloc),
            });

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items, sep)
    }
}

// Decl
//
//

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Ctor(Ctor),
    Dtor(Dtor),
    Def(Def),
    Codef(Codef),
    Let(Let),
}

impl Decl {
    pub fn kind(&self) -> DeclKind {
        match self {
            Decl::Data(_) => DeclKind::Data,
            Decl::Codata(_) => DeclKind::Codata,
            Decl::Ctor(_) => DeclKind::Ctor,
            Decl::Dtor(_) => DeclKind::Dtor,
            Decl::Def(_) => DeclKind::Def,
            Decl::Codef(_) => DeclKind::Codef,
            Decl::Let(_) => DeclKind::Let,
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
            Decl::Ctor(Ctor { name, .. }) => name,
            Decl::Dtor(Dtor { name, .. }) => name,
            Decl::Let(Let { name, .. }) => name,
        }
    }
}

impl HasSpan for Decl {
    fn span(&self) -> Option<Span> {
        match self {
            Decl::Data(data) => data.span,
            Decl::Codata(codata) => codata.span,
            Decl::Ctor(ctor) => ctor.span,
            Decl::Dtor(dtor) => dtor.span,
            Decl::Def(def) => def.span,
            Decl::Codef(codef) => codef.span,
            Decl::Let(tl_let) => tl_let.span,
        }
    }
}

impl PrintInCtx for Decl {
    type Ctx = Module;

    fn print_in_ctx<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        match self {
            Decl::Data(data) => data.print_in_ctx(cfg, ctx, alloc),
            Decl::Codata(codata) => codata.print_in_ctx(cfg, ctx, alloc),
            Decl::Def(def) => def.print(cfg, alloc),
            Decl::Codef(codef) => codef.print(cfg, alloc),
            Decl::Ctor(ctor) => ctor.print(cfg, alloc),
            Decl::Dtor(dtor) => dtor.print(cfg, alloc),
            Decl::Let(tl_let) => tl_let.print(cfg, alloc),
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
    pub typ: Rc<Telescope>,
    pub ctors: Vec<Ident>,
}

impl PrintInCtx for Data {
    type Ctx = Module;

    fn print_in_ctx<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Data { span: _, doc, name, attr, typ, ctors } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(attr.print(cfg, alloc))
            .append(alloc.keyword(DATA))
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if ctors.is_empty() {
            empty_braces(alloc)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(
                    ctors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_anno()
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
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
    pub typ: Rc<Telescope>,
    pub dtors: Vec<Ident>,
}

impl PrintInCtx for Codata {
    type Ctx = Module;

    fn print_in_ctx<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Codata { span: _, doc, name, attr, typ, dtors } = self;
        if !attr.is_visible() {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(attr.print(cfg, alloc))
            .append(alloc.keyword(CODATA))
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if dtors.is_empty() {
            empty_braces(alloc)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(
                    dtors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_anno()
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
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
        let head = alloc.ctor(name).append(params.print(cfg, alloc));

        let head = if typ.is_simple() {
            head
        } else {
            head.append(print_return_type(cfg, alloc, typ)).group()
        };
        doc.append(head)
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
    pub ret_typ: Rc<Exp>,
}

impl Print for Dtor {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { span: _, doc, name, params, self_param, ret_typ } = self;

        let doc = doc.print(cfg, alloc);
        let head = if self_param.is_simple() {
            alloc.nil()
        } else {
            self_param.print(&PrintCfg { print_function_sugar: false, ..*cfg }, alloc).append(DOT)
        };
        let head = head
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group();
        doc.append(head)
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
    pub ret_typ: Rc<Exp>,
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
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group();

        let body = print_cases(cases, cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
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
            .append(alloc.ctor(name))
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
    pub typ: Rc<Exp>,
    pub body: Rc<Exp>,
}

impl Let {
    /// Returns whether the declaration is the "main" expression of the module.
    pub fn is_main(&self) -> bool {
        self.name == "main" && self.params.is_empty()
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
            .append(name)
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, typ))
            .group();

        let body = body.print(cfg, alloc).braces_anno();

        doc.append(head).append(alloc.space()).append(body)
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
                name: self.name.clone().unwrap_or_default(),
                typ: Rc::new(self.typ.to_exp()),
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

        match name {
            Some(name) => alloc
                .text(name)
                .append(COLON)
                .append(alloc.space())
                .append(typ.print(cfg, alloc))
                .parens(),
            None => typ.print(cfg, alloc),
        }
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
    /// instead of printing `n: Nat, x: Vec(Bool,n), y: Vec(Bool,n)` we want to print
    /// `n: Nat, x y: Vec(Bool,n)`. But in de Bruijn notation this list looks like
    /// `_: Nat, _ : Vec(0), _: Vec(1)`.
    ///
    /// 2) We cannot chunk two parameters if one is implicit and the other isn't, even if they have
    /// the same type. For example: `implicit a: Type, b: Type` cannot be chunked.
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Telescope { params } = self;
        let mut output = alloc.nil();
        if params.is_empty() {
            return output;
        };
        // Running stands for the type and implicitness of the current "chunk" we are building.
        let mut running: Option<(&Rc<Exp>, bool)> = None;
        for Param { implicit, name, typ } in params {
            match running {
                // We need to shift before comparing to ensure we compare the correct De-Bruijn indices
                Some((rtype, rimplicit))
                    if &rtype.shift((0, 1)) == typ && rimplicit == *implicit =>
                {
                    // We are adding another parameter of the same type.
                    output = output.append(alloc.space()).append(alloc.text(name));
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
                        output =
                            output.append(IMPLICIT).append(alloc.space()).append(alloc.text(name));
                    } else {
                        output = output.append(alloc.text(name));
                    }
                }
                None => {
                    // We are starting a new chunk and adding the very first parameter.
                    // If we are starting a chunk of implicit parameters then we also have to
                    // add the "implicit" keyword at this point.
                    if *implicit {
                        output = output.append(IMPLICIT).append(alloc.space())
                    }

                    output = output.append(alloc.text(name));
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
        let param1 =
            Param { implicit: false, name: "x".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let param2 =
            Param { implicit: false, name: "y".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(x y: Type)")
    }

    #[test]
    fn print_simple_implicit_chunk() {
        let param1 =
            Param { implicit: true, name: "x".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let param2 =
            Param { implicit: true, name: "y".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(implicit x y: Type)")
    }

    #[test]
    fn print_mixed_implicit_chunk_1() {
        let param1 =
            Param { implicit: true, name: "x".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let param2 =
            Param { implicit: false, name: "y".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(implicit x: Type, y: Type)")
    }

    #[test]
    fn print_mixed_implicit_chunk_2() {
        let param1 =
            Param { implicit: false, name: "x".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let param2 =
            Param { implicit: true, name: "y".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let tele = Telescope { params: vec![param1, param2] };
        assert_eq!(tele.print_to_string(Default::default()), "(x: Type, implicit y: Type)")
    }

    #[test]
    fn print_shifting_example() {
        let param1 =
            Param { implicit: false, name: "a".to_owned(), typ: Rc::new(TypeUniv::new().into()) };
        let param2 = Param {
            implicit: false,
            name: "x".to_owned(),
            typ: Rc::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: 0 },
                name: "a".to_owned(),
                inferred_type: None,
            })),
        };
        let param3 = Param {
            implicit: false,
            name: "y".to_owned(),
            typ: Rc::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: 1 },
                name: "a".to_owned(),
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
    pub typ: Rc<Exp>,
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
                .append(name)
                .append(COLON)
                .append(alloc.space())
                .append(typ.print(cfg, alloc))
        } else {
            alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
        }
    }
}

// Items
//
//
pub struct Items {
    pub items: Vec<Decl>,
}

impl PrintInCtx for Items {
    type Ctx = Module;

    fn print_in_ctx<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Items { items } = self;

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items.iter().map(|item| item.print_in_ctx(cfg, ctx, alloc)), sep)
    }
}
