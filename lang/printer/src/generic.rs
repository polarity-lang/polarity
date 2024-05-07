use std::rc::Rc;

use pretty::DocAllocator;

use syntax::common::*;
use syntax::generic::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

/// Checks whether the `#[omit_print]` attribute is present.
fn is_visible(attr: &Attribute) -> bool {
    !attr.attrs.contains(&"omit_print".to_owned())
}

impl<'a> Print<'a> for Module {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let items =
            self.iter().filter(|item| is_visible(item.attributes())).map(|item| match item {
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

impl<'a> PrintInCtx<'a> for Decl {
    type Ctx = Module;

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
            Decl::Let(tl_let) => tl_let.print(cfg, alloc),
        }
    }
}

impl<'a> PrintInCtx<'a> for Item<'a> {
    type Ctx = Module;

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
            Item::Let(tl_let) => tl_let.print(cfg, alloc),
        }
    }
}

impl<'a> PrintInCtx<'a> for Data {
    type Ctx = Module;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Data { span: _, doc, name, attr, typ, ctors } = self;
        if !is_visible(attr) {
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

impl<'a> PrintInCtx<'a> for Codata {
    type Ctx = Module;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Codata { span: _, doc, name, attr, typ, dtors } = self;
        if !is_visible(attr) {
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

impl<'a> Print<'a> for Def {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { span: _, doc, name, attr, params, self_param, ret_typ, body } = self;
        if !is_visible(attr) {
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

        let body = body.print(cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl<'a> Print<'a> for Codef {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { span: _, doc, name, attr, params, typ, body } = self;
        if !is_visible(attr) {
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

        let body = body.print(cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl<'a> Print<'a> for Let {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Let { span: _, doc, name, attr, params, typ, body } = self;
        if !is_visible(attr) {
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

impl<'a> Print<'a> for Ctor {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a> Print<'a> for Dtor {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a> Print<'a> for Match {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { span: _, cases, omit_absurd } = self;
        match cases.len() {
            0 => {
                if *omit_absurd {
                    alloc
                        .space()
                        .append(alloc.text(".."))
                        .append(alloc.keyword(ABSURD))
                        .append(alloc.space())
                        .braces_anno()
                } else {
                    empty_braces(alloc)
                }
            }
            1 if !omit_absurd => alloc
                .line()
                .append(cases[0].print(cfg, alloc))
                .nest(cfg.indent)
                .append(alloc.line())
                .braces_anno()
                .group(),
            _ => {
                let sep = alloc.text(COMMA).append(alloc.hardline());
                alloc
                    .hardline()
                    .append(
                        alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep.clone()),
                    )
                    .append(if *omit_absurd {
                        sep.append(alloc.text("..")).append(alloc.keyword(ABSURD))
                    } else {
                        alloc.nil()
                    })
                    .nest(cfg.indent)
                    .append(alloc.hardline())
                    .braces_anno()
            }
        }
    }
}

impl<'a> Print<'a> for Case {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, name, params, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.ctor(name).append(params.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a> Print<'a> for Telescope {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Telescope { params } = self;
        let mut output = alloc.nil();
        if params.is_empty() {
            return output;
        };
        let mut running_type: Option<&Rc<Exp>> = None;
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

impl<'a> Print<'a> for Args {
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

impl<'a> Print<'a> for TelescopeInst {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
    }
}

impl<'a> Print<'a> for Param {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for ParamInst {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { span: _, info: _, name, typ: _ } = self;
        alloc.text(name)
    }
}

impl<'a> Print<'a> for SelfParam {
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

impl<'a> Print<'a> for Exp {
    fn print_prec(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>, prec: Precedence) -> Builder<'a> {
        match self {
            Exp::Variable(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypCtor(e) => e.print_prec(cfg, alloc, prec),
            Exp::Call(e) => e.print_prec(cfg, alloc, prec),
            mut dtor @ Exp::DotCall(DotCall { .. }) => {
                // A series of destructors forms an aligned group
                let mut dtors_group = alloc.nil();
                while let Exp::DotCall(DotCall { exp, name, args, .. }) = &dtor {
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
            Exp::Anno(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypeUniv(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalMatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalComatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::Hole(e) => e.print_prec(cfg, alloc, prec),
        }
    }
}

impl<'a> Print<'a> for Variable {
    fn print_prec(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Variable { name, idx, .. } = self;
        if cfg.de_bruijn {
            alloc.text(format!("{name}@{idx}"))
        } else {
            alloc.text(name)
        }
    }
}

impl<'a> Print<'a> for TypCtor {
    fn print_prec(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>, prec: Precedence) -> Builder<'a> {
        let TypCtor { span: _, name, args } = self;
        if name == "Fun" && args.len() == 2 && cfg.print_function_sugar {
            let arg = args.args[0].print_prec(cfg, alloc, 1);
            let res = args.args[1].print_prec(cfg, alloc, 0);
            let fun = arg.append(alloc.space()).append(ARROW).append(alloc.space()).append(res);
            if prec == 0 {
                fun
            } else {
                fun.parens()
            }
        } else {
            let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
            alloc.typ(name).append(psubst)
        }
    }
}

impl<'a> Print<'a> for Call {
    fn print_prec(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Call { name, args, .. } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
        alloc.ctor(name).append(psubst)
    }
}

impl<'a> Print<'a> for syntax::generic::Anno {
    fn print_prec(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let syntax::generic::Anno { exp, typ, .. } = self;
        exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for TypeUniv {
    fn print_prec(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.keyword(TYPE)
    }
}

impl<'a> Print<'a> for Hole {
    fn print_prec(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.keyword(HOLE)
    }
}

impl<'a> Print<'a> for LocalMatch {
    fn print_prec(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalMatch { name, on_exp, motive, body, .. } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(match &name.user_name {
                Some(name) => alloc.space().append(alloc.dtor(name)),
                None => alloc.nil(),
            })
            .append(motive.as_ref().map(|m| m.print(cfg, alloc)).unwrap_or(alloc.nil()))
            .append(alloc.space())
            .append(body.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for LocalComatch {
    fn print_prec(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalComatch { name, is_lambda_sugar, body, .. } = self;
        if *is_lambda_sugar && cfg.print_lambda_sugar {
            print_lambda_sugar(body, cfg, alloc)
        } else {
            alloc
                .keyword(COMATCH)
                .append(match &name.user_name {
                    Some(name) => alloc.space().append(alloc.ctor(name)),
                    None => alloc.nil(),
                })
                .append(alloc.space())
                .append(body.print(cfg, alloc))
        }
    }
}

impl<'a> Print<'a> for Motive {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Motive { span: _, param, ret_typ } = self;

        alloc
            .space()
            .append(alloc.keyword(AS))
            .append(alloc.space())
            .append(param.print(cfg, alloc))
            .append(alloc.space())
            .append(alloc.text(FAT_ARROW))
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for Attribute {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.attrs.is_empty() {
            alloc.nil()
        } else {
            let p = print_comma_separated(&self.attrs, cfg, alloc);
            alloc.text(HASH).append(p.brackets()).append(alloc.hardline())
        }
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

    fn print_prec(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>, prec: Precedence) -> Builder<'a> {
        T::print_prec(self, cfg, alloc, prec)
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
fn print_lambda_sugar<'a>(e: &'a Match, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    let Match { cases, .. } = e;
    let Case { params, body, .. } = cases.first().expect("Empty comatch marked as lambda sugar");
    let var_name = params
        .params
        .get(2) // The variable we want to print is at the third position: comatch { ap(_,_,x) => ...}
        .expect("No parameter bound in comatch marked as lambda sugar")
        .name();
    alloc
        .backslash_anno(cfg)
        .append(var_name)
        .append(DOT)
        .append(alloc.space())
        .append(body.print(cfg, alloc))
}

fn print_comma_separated<'a, T: Print<'a>>(
    vec: &'a [T],
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
fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
}
