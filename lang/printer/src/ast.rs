use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;

impl<'a, P: Phase> Print<'a> for Prg<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Prg { decls, exp } = self;

        match exp {
            Some(exp) => {
                let top = if decls.is_empty() {
                    alloc.nil()
                } else {
                    decls.print(cfg, alloc).append(alloc.hardline()).append(alloc.hardline())
                };
                top.append(exp.print(cfg, alloc))
            }
            None => decls.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> Print<'a> for Decls<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Decls { map, order } = self;
        let decls_in_order = order
            .iter()
            .map(|name| &map[name])
            .filter(|x| matches!(x, Decl::Data(_) | Decl::Codata(_)))
            .map(|x| x.print_in_ctx(cfg, self, alloc));
        let sep = alloc.line().append(alloc.line());
        alloc.intersperse(decls_in_order, sep)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Decl<P> {
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        match self {
            Decl::Data(data) => {
                let impl_block = &data.impl_block;
                let data = data.print_in_ctx(cfg, ctx, alloc);

                match impl_block {
                    Some(block) => data
                        .append(alloc.hardline())
                        .append(alloc.hardline())
                        .append(block.print_in_ctx(cfg, ctx, alloc)),
                    None => data,
                }
            }
            Decl::Codata(codata) => {
                let impl_block = &codata.impl_block;
                let codata = codata.print_in_ctx(cfg, ctx, alloc);

                match impl_block {
                    Some(block) => codata
                        .append(alloc.hardline())
                        .append(alloc.hardline())
                        .append(block.print_in_ctx(cfg, ctx, alloc)),
                    None => codata,
                }
            }
            Decl::Def(def) => def.print(cfg, alloc),
            Decl::Codef(codef) => codef.print(cfg, alloc),
            Decl::Ctor(ctor) => ctor.print(cfg, alloc),
            Decl::Dtor(dtor) => dtor.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Data<P> {
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Data { info: _, name, typ, ctors, impl_block: _ } = self;

        let head = alloc
            .keyword(DATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body =
            alloc
                .hardline()
                .append(alloc.intersperse(
                    ctors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(INDENT)
                .append(alloc.hardline())
                .braces_from(cfg);

        head.append(body)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Codata<P> {
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Codata { info: _, name, typ, dtors, impl_block: _ } = self;
        let head = alloc
            .keyword(CODATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body =
            alloc
                .hardline()
                .append(alloc.intersperse(
                    dtors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(INDENT)
                .append(alloc.hardline())
                .braces_from(cfg);

        head.append(body)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Impl<P> {
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Impl { info: _, name, defs } = self;

        let head =
            alloc.keyword(IMPL).append(alloc.space()).append(alloc.typ(name)).append(alloc.space());

        let sep = alloc.hardline().append(alloc.hardline());

        let body =
            alloc
                .hardline()
                .append(alloc.intersperse(
                    defs.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(INDENT)
                .append(alloc.hardline())
                .braces_from(cfg);

        head.append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Def<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { info: _, name, params, on_typ, in_typ, body } = self;
        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(on_typ.print(cfg, alloc))
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(in_typ.print(cfg, alloc));

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Codef<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { info: _, name, params, typ, body } = self;
        let head = alloc
            .keyword(CODEF)
            .append(alloc.space())
            .append(alloc.ctor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(cfg, alloc));

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Ctor<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Ctor { info: _, name, params, typ } = self;
        alloc
            .ctor(name)
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for Dtor<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { info: _, name, params, on_typ, in_typ } = self;
        on_typ
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(in_typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for Comatch<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());

        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(INDENT)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a, P: Phase> Print<'a> for Match<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(INDENT)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a, P: Phase> Print<'a> for Case<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { info: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(INDENT),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a, P: Phase> Print<'a> for Cocase<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(INDENT),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a, P: Phase> Print<'a> for Telescope<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        self.params.print(cfg, alloc).parens()
    }
}

impl<'a, P: Phase> Print<'a> for Param<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for TypApp<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { info: _, name, args: subst } = self;
        alloc.typ(name).append(subst.print(cfg, alloc).parens())
    }
}

impl<'a, P: Phase> Print<'a> for Exp<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { info: _, name, idx } => alloc.text(P::print_var(name, *idx)),
            Exp::TypCtor { info: _, name, args: subst } => {
                alloc.typ(name).append(subst.print(cfg, alloc).parens())
            }
            Exp::Ctor { info: _, name, args: subst } => {
                alloc.ctor(name).append(subst.print(cfg, alloc).parens())
            }
            Exp::Dtor { info: _, exp, name, args: subst } => exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.dtor(name))
                .append(subst.print(cfg, alloc).parens()),
            Exp::Anno { info: _, exp, typ } => {
                exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
            }
            Exp::Type { info: _ } => alloc.typ(TYPE),
        }
    }
}

impl<'a, T: Print<'a>> Print<'a> for Rc<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }
}

impl<'a, T: Print<'a>> Print<'a> for Vec<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.is_empty() {
            alloc.nil()
        } else {
            let sep = alloc.text(COMMA).append(alloc.space());
            alloc.intersperse(self.iter().map(|x| x.print(cfg, alloc)), sep)
        }
    }
}

trait BracesExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A>;
}

impl<'a, D, A> BracesExt<'a, D, A> for pretty::DocBuilder<'a, D, A>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A> {
        self.enclose(cfg.braces.0, cfg.braces.1)
    }
}
