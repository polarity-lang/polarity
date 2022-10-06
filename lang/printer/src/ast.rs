use std::rc::Rc;

use pretty::DocAllocator;

use syntax::ast::*;

use super::theme::ThemeExt;
use super::tokens::*;
use super::types::*;

impl<'a> Print<'a> for Prg {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Prg { decls, exp } = self;

        match exp {
            Some(exp) => decls.print(alloc).append(exp.print(alloc)),
            None => decls.print(alloc),
        }
    }
}

impl<'a> Print<'a> for Decls {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Decls { map, order } = self;
        let decls_in_order = order
            .iter()
            .map(|name| &map[name])
            .filter(|x| !matches!(x, Decl::Ctor(_) | Decl::Dtor(_)))
            .map(|x| x.print_in_ctx(self, alloc));
        let sep = alloc.text(SEMI).append(alloc.line()).append(alloc.line());
        alloc.intersperse(decls_in_order, sep)
    }
}

impl<'a> PrintInCtx<'a> for Decl {
    type Ctx = Decls;

    fn print_in_ctx(&'a self, ctx: &'a Self::Ctx, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Decl::Data(data) => data.print_in_ctx(ctx, alloc),
            Decl::Codata(codata) => codata.print_in_ctx(ctx, alloc),
            Decl::Def(def) => def.print(alloc),
            Decl::Codef(codef) => codef.print(alloc),
            Decl::Ctor(ctor) => ctor.print(alloc),
            Decl::Dtor(dtor) => dtor.print(alloc),
        }
    }
}

impl<'a> PrintInCtx<'a> for Data {
    type Ctx = Decls;

    fn print_in_ctx(&'a self, ctx: &'a Self::Ctx, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Data { info: _, name, typ, ctors } = self;

        let head = alloc
            .keyword(DATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space())
            .append(COLON_EQ);

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body = alloc
            .hardline()
            .append(
                alloc.intersperse(ctors.iter().map(|x| ctx.map[x].print_in_ctx(ctx, alloc)), sep),
            )
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> PrintInCtx<'a> for Codata {
    type Ctx = Decls;

    fn print_in_ctx(&'a self, ctx: &'a Self::Ctx, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codata { info: _, name, typ, dtors } = self;
        let head = alloc
            .keyword(CODATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(typ.params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space())
            .append(COLON_EQ);

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body = alloc
            .hardline()
            .append(
                alloc.intersperse(dtors.iter().map(|x| ctx.map[x].print_in_ctx(ctx, alloc)), sep),
            )
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> Print<'a> for Def {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { info: _, name, params, on_typ, in_typ, body } = self;
        let head = alloc
            .keyword(DEF)
            .append(alloc.space())
            .append(on_typ.print(alloc))
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(in_typ.print(alloc))
            .append(alloc.space())
            .append(COLON_EQ);

        let body = body.print(alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a> Print<'a> for Codef {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codef { info: _, name, params, typ, body } = self;
        let head = alloc
            .keyword(CODEF)
            .append(alloc.space())
            .append(alloc.ctor(name))
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(alloc))
            .append(alloc.space())
            .append(COLON_EQ);

        let body = body.print(alloc);

        head.append(alloc.space()).append(body)
    }
}

impl<'a> Print<'a> for Ctor {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Ctor { info: _, name, params, typ } = self;
        alloc
            .ctor(name)
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(typ.print(alloc))
    }
}

impl<'a> Print<'a> for Dtor {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Dtor { info: _, name, params, on_typ, in_typ } = self;
        on_typ
            .print(alloc)
            .append(DOT)
            .append(alloc.dtor(name))
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(in_typ.print(alloc))
    }
}

impl<'a> Print<'a> for Comatch {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        let head = alloc.keyword(COMATCH);

        let sep = alloc.text(COMMA).append(alloc.hardline());
        let body = alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(alloc)), sep))
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> Print<'a> for Match {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        let head = alloc.keyword(MATCH);

        let sep = alloc.text(COMMA).append(alloc.hardline());
        let body = alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(alloc)), sep))
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> Print<'a> for Case {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { info: _, name, args, eqns, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => body.print(alloc),
        };

        alloc
            .ctor(name)
            .append(args.print(alloc))
            .append(eqns.print(alloc).braces())
            .append(alloc.space())
            .append(FAT_ARROW)
            .append(alloc.line().append(body).nest(INDENT))
            .group()
    }
}

impl<'a> Print<'a> for Cocase {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, args, eqns, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => body.print(alloc),
        };

        alloc
            .ctor(name)
            .append(args.print(alloc))
            .append(eqns.print(alloc).braces())
            .append(alloc.space())
            .append(FAT_ARROW)
            .append(alloc.line().append(body).nest(INDENT))
            .group()
    }
}

impl<'a> Print<'a> for Telescope {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        self.0.print(alloc).parens()
    }
}

impl<'a> Print<'a> for Param {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(alloc))
    }
}

impl<'a> Print<'a> for EqnParam {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let EqnParam { name, eqn } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(eqn.print(alloc))
    }
}

impl<'a> Print<'a> for TypApp {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { info: _, name, args: subst } = self;
        alloc.typ(name).append(subst.print(alloc).parens())
    }
}

impl<'a> Print<'a> for Eqn {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Eqn { info: _, lhs, rhs } = self;
        lhs.print(alloc)
            .append(alloc.space())
            .append(EQ)
            .append(alloc.space())
            .append(rhs.print(alloc))
    }
}

impl<'a> Print<'a> for Exp {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { info: _, idx } => idx.print(alloc),
            Exp::TypCtor { info: _, name, args: subst } => {
                alloc.typ(name).append(subst.print(alloc).parens())
            }
            Exp::Ctor { info: _, name, args: subst } => {
                alloc.ctor(name).append(subst.print(alloc).parens())
            }
            Exp::Dtor { info: _, exp, name, args: subst } => exp
                .print(alloc)
                .append(DOT)
                .append(alloc.dtor(name))
                .append(subst.print(alloc).parens()),
            Exp::Anno { info: _, exp, typ } => {
                exp.print(alloc).parens().append(COLON).append(typ.print(alloc))
            }
            Exp::Type { info: _ } => alloc.typ(TYPE),
        }
    }
}

impl<'a, T: Print<'a>> Print<'a> for Rc<T> {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, alloc)
    }
}

impl<'a, T: Print<'a>> Print<'a> for Vec<T> {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.is_empty() {
            alloc.nil()
        } else {
            let sep = alloc.text(COMMA).append(alloc.space());
            alloc.intersperse(self.iter().map(|x| x.print(alloc)), sep)
        }
    }
}
