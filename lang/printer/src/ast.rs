use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

impl<'a, P: Phase> Print<'a> for Prg<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Prg { decls, exp } = self;

        match exp {
            Some(exp) => {
                let top = if decls.is_empty() {
                    alloc.nil()
                } else {
                    let sep = if cfg.omit_decl_sep {
                        alloc.hardline()
                    } else {
                        alloc.hardline().append(alloc.hardline())
                    };
                    decls.print(cfg, alloc).append(sep)
                };
                top.append(exp.print(cfg, alloc))
            }
            None => decls.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> Print<'a> for Decls<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let items = self.iter().map(|item| match item {
            Item::Data(data) => data.print_in_ctx(cfg, self, alloc),
            Item::Codata(codata) => codata.print_in_ctx(cfg, self, alloc),
            Item::Impl(impl_block) => impl_block.print_in_ctx(cfg, self, alloc),
        });

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items, sep)
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
            Decl::Data(data) => data.print_in_ctx(cfg, ctx, alloc),
            Decl::Codata(codata) => codata.print_in_ctx(cfg, ctx, alloc),
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
        let Def { info: _, name, params, self_param, ret_typ, body } = self;
        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(self_param.print(cfg, alloc))
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc));

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
        let Dtor { info: _, name, params, self_param, ret_typ } = self;
        self_param
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc))
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
        let Cocase { info: _, name, params: args, body } = self;

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
        print_comma_separated(&self.params, cfg, alloc).opt_parens()
    }
}

impl<'a, P: Phase> Print<'a> for TelescopeInst<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        self.params.print(cfg, alloc).opt_parens()
    }
}

impl<'a, P: Phase> Print<'a> for Param<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for ParamInst<P> {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { info: _, name, typ: _ } = self;
        alloc.text(name)
    }
}

impl<'a, P: Phase> Print<'a> for SelfParam<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a, P: Phase> Print<'a> for TypApp<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { info: _, name, args: subst } = self;
        alloc.typ(name).append(subst.print(cfg, alloc).opt_parens())
    }
}

impl<'a, P: Phase> Print<'a> for Exp<P> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { info: _, name, idx } => {
                alloc.text(P::print_var(name, cfg.de_bruijn.then_some(*idx)))
            }
            Exp::TypCtor { info: _, name, args: subst } => {
                alloc.typ(name).append(subst.print(cfg, alloc).opt_parens())
            }
            Exp::Ctor { info: _, name, args: subst } => {
                alloc.ctor(name).append(subst.print(cfg, alloc).opt_parens())
            }
            Exp::Dtor { info: _, exp, name, args: subst } => exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.dtor(name))
                .append(subst.print(cfg, alloc).opt_parens()),
            Exp::Anno { info: _, exp, typ } => {
                exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
            }
            Exp::Type { info: _ } => alloc.typ(TYPE),
            Exp::Match { info: _, name, on_exp, ret_typ: _, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Exp::Comatch { info: _, name, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
        }
    }
}

impl<'a, T: Print<'a>> Print<'a> for Rc<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }
}

fn print_comma_separated<'a, T: Print<'a>>(
    vec: &'a Vec<T>,
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
) -> Builder<'a> {
    if vec.is_empty() {
        alloc.nil()
    } else {
        let sep = alloc.text(COMMA).append(alloc.space());
        alloc.intersperse(vec.iter().map(|x| x.print(cfg, alloc)), sep)
    }
}

impl<'a, T: Print<'a>> Print<'a> for Vec<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        print_comma_separated(self, cfg, alloc)
    }
}
