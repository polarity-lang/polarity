use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;
use super::util::*;

/// Checks whether the `#[omit_print]` attribute is present.
fn is_visible(attr: &Attributes) -> bool {
    !attr.attrs.contains(&Attribute::OmitPrint)
}

impl Print for Module {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'b> PrintInCtx for Item<'b> {
    type Ctx = Module;

    fn print_in_ctx<'a>(
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

impl PrintInCtx for Data {
    type Ctx = Module;

    fn print_in_ctx<'a>(
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

impl PrintInCtx for Codata {
    type Ctx = Module;

    fn print_in_ctx<'a>(
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

impl Print for Def {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { span: _, doc, name, attr, params, self_param, ret_typ, cases } = self;
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

        let body = print_cases(cases, cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl Print for Codef {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { span: _, doc, name, attr, params, typ, cases } = self;
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

        let body = print_cases(cases, cfg, alloc);

        doc.append(head).append(alloc.space()).append(body)
    }
}

impl Print for Let {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

fn print_cases<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    match cases.len() {
        0 => empty_braces(alloc),

        1 => alloc
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
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep.clone()))
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_anno()
        }
    }
}

impl Print for Case {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

    use syntax::common::Idx;

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

impl Print for Args {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl Print for TelescopeInst {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
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

impl Print for ParamInst {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { span: _, info: _, name, typ: _ } = self;
        alloc.text(name)
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

impl Print for Exp {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
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

impl Print for Variable {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Variable { name, idx, .. } = self;
        if cfg.de_bruijn {
            alloc.text(format!("{name}@{idx}"))
        } else if name.is_empty() {
            alloc.text(format!("@{idx}"))
        } else {
            alloc.text(name)
        }
    }
}

impl Print for TypCtor {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
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

impl Print for Call {
    fn print_prec<'a>(
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

impl Print for syntax::ast::Anno {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let syntax::ast::Anno { exp, typ, .. } = self;
        exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
    }
}

impl Print for TypeUniv {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.keyword(TYPE)
    }
}

impl Print for Hole {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        if cfg.print_metavar_ids {
            alloc.text(format!("?{}", self.metavar.id))
        } else {
            alloc.keyword(HOLE)
        }
    }
}

impl Print for LocalMatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalMatch { name, on_exp, motive, cases, .. } = self;
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
            .append(print_cases(cases, cfg, alloc))
    }
}

impl Print for LocalComatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalComatch { name, is_lambda_sugar, cases, .. } = self;
        if *is_lambda_sugar && cfg.print_lambda_sugar {
            print_lambda_sugar(cases, cfg, alloc)
        } else {
            alloc
                .keyword(COMATCH)
                .append(match &name.user_name {
                    Some(name) => alloc.space().append(alloc.ctor(name)),
                    None => alloc.nil(),
                })
                .append(alloc.space())
                .append(print_cases(cases, cfg, alloc))
        }
    }
}

impl Print for Motive {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<T: Print> Print for Rc<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }

    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        T::print_prec(self, cfg, alloc, prec)
    }
}

impl<T: Print> Print for Option<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Some(inner) => inner.print(cfg, alloc),
            None => alloc.nil(),
        }
    }
}

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

/// Print the Comatch as a lambda abstraction.
/// Only invoke this function if the comatch contains exactly
/// one cocase "ap" with three arguments; the function will
/// panic otherwise.
fn print_lambda_sugar<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

fn print_comma_separated<'a, T: Print>(
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

impl<T: Print> Print for Vec<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        print_comma_separated(self, cfg, alloc)
    }
}

// Prints "{ }"
fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
}
