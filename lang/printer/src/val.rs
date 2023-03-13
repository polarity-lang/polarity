use pretty::DocAllocator;

use syntax::val::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

impl<'a> Print<'a> for Val {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Val::TypCtor { info: _, name, args: subst } => {
                let psubst =
                    if subst.is_empty() { alloc.nil() } else { subst.print(cfg, alloc).parens() };
                alloc.typ(name).append(psubst)
            }
            Val::Ctor { info: _, name, args: subst } => {
                let psubst =
                    if subst.is_empty() { alloc.nil() } else { subst.print(cfg, alloc).parens() };
                alloc.ctor(name).append(psubst)
            }
            Val::Type { info: _ } => alloc.typ(TYPE),
            Val::Comatch { info: _, name, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Val::Neu { exp } => exp.print(cfg, alloc),
        }
    }
}

impl<'a> Print<'a> for Neu {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Var { info: _, name, idx } => alloc.text(format!("{name}@{idx}")),
            Neu::Dtor { info: _, exp, name, args: subst } => {
                let psubst =
                    if subst.is_empty() { alloc.nil() } else { subst.print(cfg, alloc).parens() };
                exp.print(cfg, alloc).append(DOT).append(alloc.dtor(name)).append(psubst)
            }
            Neu::Match { info: _, name, on_exp, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Neu::Hole { info: _, kind } => match kind {
                syntax::common::HoleKind::Todo => alloc.keyword(HOLE_TODO),
                syntax::common::HoleKind::Omitted => alloc.keyword(HOLE_OMITTED),
            },
        }
    }
}
impl<'a> Print<'a> for Comatch {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());

        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a> Print<'a> for Match {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a> Print<'a> for Case {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { info: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a> Print<'a> for Cocase {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a> Print<'a> for Closure {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text("...")
    }
}
