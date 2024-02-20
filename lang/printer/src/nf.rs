use pretty::DocAllocator;

use syntax::nf::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

impl<'a> Print<'a> for Nf {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Nf::TypCtor { info: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.typ(name).append(psubst)
            }
            Nf::Ctor { info: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.ctor(name).append(psubst)
            }
            Nf::Type { info: _ } => alloc.typ(TYPE),
            Nf::Comatch { info: _, name, is_lambda_sugar: _, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name.to_string()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Nf::Neu { exp } => exp.print(cfg, alloc),
        }
    }
}

impl<'a> Print<'a> for Neu {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Var { info: _, name, idx } => alloc.text(format!("{name}@{idx}")),
            Neu::Dtor { info: _, exp, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                exp.print(cfg, alloc).append(DOT).append(alloc.dtor(name)).append(psubst)
            }
            Neu::Match { info: _, name, on_exp, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name.to_string()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Neu::Hole { .. } => alloc.keyword(HOLE_TODO),
        }
    }
}

impl<'a> Print<'a> for Match {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases, omit_absurd } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .append(if *omit_absurd {
                alloc.text(COMMA).append(alloc.text("..")).append(alloc.keyword(ABSURD))
            } else {
                alloc.nil()
            })
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_anno()
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
