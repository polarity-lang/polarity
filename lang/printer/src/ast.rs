use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;
use syntax::common::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

impl<'a, P: Phase> Print<'a> for Prg<P>
where
    P::InfTyp: ShiftInRange,
{
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

impl<'a, P: Phase> Print<'a> for Decls<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let items = self.iter().filter(|item| !item.hidden()).map(|item| match item {
            Item::Data(data) => data.print_in_ctx(cfg, self, alloc),
            Item::Codata(codata) => codata.print_in_ctx(cfg, self, alloc),
            Item::Def(def) => def.print(cfg, alloc),
            Item::Codef(codef) => codef.print(cfg, alloc),
        });

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items, sep)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Decl<P>
where
    P::InfTyp: ShiftInRange,
{
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

impl<'a, P: Phase> PrintInCtx<'a> for Item<'a, P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        match self {
            Item::Data(data) => data.print_in_ctx(cfg, ctx, alloc),
            Item::Codata(codata) => codata.print_in_ctx(cfg, ctx, alloc),
            Item::Def(def) => def.print(cfg, alloc),
            Item::Codef(codef) => codef.print(cfg, alloc),
        }
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Data<P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Data { info: _, doc, name, hidden, typ, ctors } = self;
        if *hidden {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(alloc.keyword(DATA))
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if ctors.is_empty() {
            empty_braces(alloc, cfg)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(
                    ctors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_from(cfg)
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
    }
}

impl<'a, P: Phase> PrintInCtx<'a> for Codata<P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Codata { info: _, doc, name, hidden, typ, dtors } = self;
        if *hidden {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(alloc.keyword(CODATA))
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(cfg, alloc))
            .append(alloc.space());

        let sep = alloc.text(COMMA).append(alloc.line());

        let body = if dtors.is_empty() {
            empty_braces(alloc, cfg)
        } else {
            alloc
                .line()
                .append(alloc.intersperse(
                    dtors.iter().map(|x| ctx.map[x].print_in_ctx(cfg, ctx, alloc)),
                    sep,
                ))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_from(cfg)
        };

        let body = if typ.params.is_empty() { body.group() } else { body };

        head.append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Def<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { info: _, doc, name, hidden, params, self_param, ret_typ, body } = self;
        if *hidden {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(alloc.keyword(DEF))
            .append(alloc.space())
            .append(self_param.print(cfg, alloc))
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group();

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Codef<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { info: _, doc, name, hidden, params, typ, body } = self;
        if *hidden {
            return alloc.nil();
        }

        let head = doc
            .print(cfg, alloc)
            .append(alloc.keyword(CODEF))
            .append(alloc.space())
            .append(alloc.ctor(name))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, typ))
            .group();

        let body = body.print(cfg, alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a, P: Phase> Print<'a> for Ctor<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Ctor { info: _, doc, name, params, typ } = self;

        let head = doc.print(cfg, alloc).append(alloc.ctor(name)).append(params.print(cfg, alloc));

        if typ.is_simple() {
            head
        } else {
            head.append(print_return_type(cfg, alloc, typ)).group()
        }
    }
}

impl<'a, P: Phase> Print<'a> for Dtor<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { info: _, doc, name, params, self_param, ret_typ } = self;

        let head = if self_param.is_simple() {
            doc.print(cfg, alloc)
        } else {
            doc.print(cfg, alloc).append(self_param.print(cfg, alloc)).append(DOT)
        };
        head.append(alloc.dtor(name))
            .append(params.print(cfg, alloc))
            .append(print_return_type(cfg, alloc, ret_typ))
            .group()
    }
}

impl<'a, P: Phase> Print<'a> for Comatch<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        if cases.is_empty() {
            empty_braces(alloc, cfg)
        } else {
            let sep = alloc.text(COMMA).append(alloc.hardline());

            alloc
                .hardline()
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_from(cfg)
        }
    }
}

impl<'a, P: Phase> Print<'a> for Match<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        if cases.is_empty() {
            empty_braces(alloc, cfg)
        } else {
            let sep = alloc.text(COMMA).append(alloc.hardline());
            alloc
                .hardline()
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_from(cfg)
        }
    }
}

impl<'a, P: Phase> Print<'a> for Case<P>
where
    P::InfTyp: ShiftInRange,
{
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

impl<'a, P: Phase> Print<'a> for Cocase<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, params: args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.dtor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a, P: Phase> Print<'a> for Telescope<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Telescope { params } = self;
        let mut output = alloc.nil();
        if params.is_empty() {
            return output;
        };
        let mut running_type: Option<&Rc<Exp<P>>> = None;
        for Param { name, typ } in params {
            match running_type {
                // We need to shift before comparing to ensure we compare the correct De-Bruijn indices
                Some(rtype) if &rtype.shift((0, 1)) == typ => {
                    // We are adding another parameter of the same type.
                    output = output.append(alloc.space()).append(alloc.text(name));
                }
                Some(rtype) => {
                    // We are adding another parameter with a different type,
                    // and have to close the previous list first.
                    output = output
                        .append(COLON)
                        .append(alloc.space())
                        .append(rtype.print(cfg, alloc))
                        .append(COMMA)
                        .append(alloc.line());
                    output = output.append(alloc.text(name));
                }
                None => {
                    // We are adding the very first parameter.
                    output = output.append(alloc.text(name));
                }
            }
            running_type = Some(typ);
        }
        // Close the last parameter
        match running_type {
            None => {}
            Some(rtype) => {
                output = output.append(COLON).append(alloc.space()).append(rtype.print(cfg, alloc));
            }
        }
        output.append(alloc.line_()).align().parens().group()
    }
}

impl<'a, P: Phase> Print<'a> for Args<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let mut doc = alloc.nil();
        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            doc = doc.append(arg.print(cfg, alloc));
            if iter.peek().is_some() {
                doc = doc.append(COMMA).append(alloc.line())
            }
        }
        doc.align().parens().group()
    }
}

impl<'a, P: Phase> Print<'a> for TelescopeInst<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
    }
}

impl<'a, P: Phase> Print<'a> for Param<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
    }
}

impl<'a, P: Phase> Print<'a> for ParamInst<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { info: _, name, typ: _ } = self;
        alloc.text(name)
    }
}

impl<'a, P: Phase> Print<'a> for SelfParam<P>
where
    P::InfTyp: ShiftInRange,
{
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

impl<'a, P: Phase> Print<'a> for TypApp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { info: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
        alloc.typ(name).append(psubst)
    }
}

impl<'a, P: Phase> Print<'a> for Exp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { info: _, name, idx } => {
                alloc.text(P::print_var(name, cfg.de_bruijn.then_some(*idx)))
            }
            Exp::TypCtor { info: _, name, args } => {
                let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
                alloc.typ(name).append(psubst)
            }
            Exp::Ctor { info: _, name, args } => {
                let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
                alloc.ctor(name).append(psubst)
            }
            mut dtor @ Exp::Dtor { .. } => {
                // A series of destructors forms an aligned group
                let mut dtors_group = alloc.nil();
                while let Exp::Dtor { info: _, exp, name, args } = dtor {
                    let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
                    if !dtors_group.is_nil() {
                        dtors_group = alloc.line_().append(dtors_group);
                    }
                    dtors_group =
                        alloc.text(DOT).append(alloc.dtor(name)).append(psubst).append(dtors_group);
                    dtor = exp;
                }
                dtor.print(cfg, alloc).append(dtors_group.align().group())
            }
            Exp::Anno { info: _, exp, typ } => {
                exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
            }
            Exp::Type { info: _ } => alloc.keyword(TYPE),
            Exp::Match { info: _, name, on_exp, motive, ret_typ: _, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(match P::print_label(name) {
                    Some(name) => alloc.space().append(alloc.dtor(&name)),
                    None => alloc.nil(),
                })
                .append(motive.as_ref().map(|m| m.print(cfg, alloc)).unwrap_or(alloc.nil()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Exp::Comatch { info: _, name, is_lambda_sugar, body } => {
                if *is_lambda_sugar && cfg.print_lambda_sugar {
                    print_lambda_sugar(body, cfg, alloc)
                } else {
                    alloc
                        .keyword(COMATCH)
                        .append(match P::print_label(name) {
                            Some(name) => alloc.space().append(alloc.ctor(&name)),
                            None => alloc.nil(),
                        })
                        .append(alloc.space())
                        .append(body.print(cfg, alloc))
                }
            }
            Exp::Hole { info: _, kind } => match kind {
                HoleKind::Todo => alloc.keyword(HOLE_TODO),
                HoleKind::Omitted => alloc.keyword(HOLE_OMITTED),
            },
        }
    }
}

impl<'a, P: Phase> Print<'a> for Motive<P>
where
    P::InfTyp: ShiftInRange,
{
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Motive { info: _, param, ret_typ } = self;

        alloc
            .space()
            .append(alloc.keyword(AS))
            .append(alloc.space())
            .append(param.print(cfg, alloc))
            .append(alloc.text(FAT_ARROW))
            .append(ret_typ.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for DocComment {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let DocComment { docs } = self;
        let prefix = "-- | ";
        alloc.concat(
            docs.iter().map(|doc| {
                alloc.comment(prefix).append(alloc.comment(doc)).append(alloc.hardline())
            }),
        )
    }
}

impl<'a, T: Print<'a>> Print<'a> for Rc<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }
}

impl<'a, T: Print<'a>> Print<'a> for Option<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Some(inner) => inner.print(cfg, alloc),
            None => alloc.nil(),
        }
    }
}

fn print_return_type<'a, T: Print<'a>>(
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

/// Print the Comatch as a lambda abstraction.
/// Only invoke this function if the comatch contains exactly
/// one cocase "ap" with three arguments; the function will
/// panic otherwise.
fn print_lambda_sugar<'a, P: Phase>(
    e: &'a Comatch<P>,
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
) -> Builder<'a>
where
    P::InfTyp: ShiftInRange,
{
    let Comatch { cases, .. } = e;
    let Cocase { params, body, .. } = cases.get(0).expect("Empty comatch marked as lambda sugar");
    let var_name = params
        .params
        .get(2) // The variable we want to print is at the third position: comatch { ap(_,_,x) => ...}
        .expect("No parameter bound in comatch marked as lambda sugar")
        .name();
    alloc
        .backslash_from(cfg)
        .append(var_name)
        .append(DOT)
        .append(alloc.space())
        .append(body.print(cfg, alloc))
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

// Prints "{ }"
fn empty_braces<'a>(alloc: &'a Alloc<'a>, cfg: &PrintCfg) -> Builder<'a> {
    alloc.space().braces_from(cfg)
}
