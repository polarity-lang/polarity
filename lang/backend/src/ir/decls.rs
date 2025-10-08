use url::Url;

use polarity_lang_ast::UseDecl;
use polarity_lang_printer::theme::ThemeExt;
use polarity_lang_printer::tokens::*;
use polarity_lang_printer::util::{BracesExt, IsNilExt};
use polarity_lang_printer::{Alloc, Builder, DocAllocator, Print, PrintCfg};

use super::exprs::{Case, Exp};
use super::exprs::{print_cases, print_params};

#[derive(Debug, Clone)]
pub struct Module {
    pub uri: Url,
    pub use_decls: Vec<UseDecl>,
    pub def_decls: Vec<Def>,
    pub codef_decls: Vec<Codef>,
    pub let_decls: Vec<Let>,
}

impl Print for Module {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Module { uri: _, use_decls, def_decls, codef_decls, let_decls } = self;

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

        let def_decls = def_decls.iter().map(|decl| decl.print(cfg, alloc));
        let codef_decls = codef_decls.iter().map(|decl| decl.print(cfg, alloc));
        let let_decls = let_decls.iter().map(|decl| decl.print(cfg, alloc));

        let decls = alloc.intersperse(def_decls.chain(codef_decls).chain(let_decls), sep);

        // UseDecls + Decls
        //
        //

        let doc = if use_decls.is_nil() {
            decls
        } else {
            use_decls.append(alloc.line()).append(alloc.line()).append(decls)
        };

        if doc.is_nil() { doc } else { doc.append(alloc.hardline()) }
    }
}

#[derive(Debug, Clone)]
pub struct Def {
    pub name: String,
    pub params: Vec<String>,
    pub cases: Vec<Case>,
}

impl Print for Def {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { name, params, cases } = self;
        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(DOT)
            .append(alloc.dtor(name))
            .append(print_params(params, alloc))
            .group();

        let body = print_cases(cases, cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub name: String,
    pub params: Vec<String>,
    pub cases: Vec<Case>,
}

impl Print for Codef {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { name, params, cases } = self;
        let head = alloc
            .keyword(CODEF)
            .append(alloc.space())
            .append(alloc.ctor(name))
            .append(print_params(params, alloc))
            .group();

        let body = print_cases(cases, cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Exp>,
}

impl Print for Let {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Let { name, params, body } = self;

        let head = alloc
            .keyword(LET)
            .append(alloc.space())
            .append(name)
            .append(print_params(params, alloc))
            .group();

        let body = alloc
            .line()
            .append(body.print(cfg, alloc))
            .nest(cfg.indent)
            .append(alloc.line())
            .braces_anno()
            .group();

        head.append(alloc.space()).append(body)
    }
}
