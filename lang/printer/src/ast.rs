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
        let decls_in_order = order.iter().map(|name| &map[name]).map(|x| x.print(alloc));
        let sep = alloc.text(SEMI).append(alloc.line()).append(alloc.line());
        alloc.intersperse(decls_in_order, sep)
    }
}

impl<'a> Print<'a> for Decl {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Decl::Data(data) => data.print(alloc),
            Decl::Codata(codata) => codata.print(alloc),
            Decl::Def(def) => def.print(alloc),
            Decl::Codef(codef) => codef.print(alloc),
        }
    }
}

impl<'a> Print<'a> for Data {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Data { name, params, ctors } = self;

        let head = alloc
            .keyword(DATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space())
            .append(COLON_EQ);

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body = alloc
            .hardline()
            .append(alloc.intersperse(ctors.iter().map(|x| x.print(alloc)), sep))
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> Print<'a> for Codata {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Codata { name, params, dtors } = self;
        let head = alloc
            .keyword(CODATA)
            .append(alloc.space())
            .append(alloc.typ(name))
            .append(params.print(alloc))
            .append(alloc.space())
            .append(COLON)
            .append(alloc.space())
            .append(alloc.typ(TYPE))
            .append(alloc.space())
            .append(COLON_EQ);

        let sep = alloc.text(COMMA).append(alloc.hardline());

        let body = alloc
            .hardline()
            .append(alloc.intersperse(dtors.iter().map(|x| x.print(alloc)), sep))
            .nest(INDENT);

        head.append(body)
    }
}

impl<'a> Print<'a> for Def {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Def { name, params, on_typ, in_typ, body } = self;
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
        let Codef { name, params, typ, body } = self;
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
        let Ctor { name, params, typ } = self;
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
        let Dtor { name, params, on_typ, in_typ } = self;
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
        let Comatch { cases } = self;
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
        let Match { cases } = self;
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
        let Case { name, args, body } = self;
        alloc
            .ctor(name)
            .append(args.print(alloc))
            .append(alloc.space())
            .append(FAT_ARROW)
            .append(alloc.line().append(body.print(alloc)).nest(INDENT))
            .group()
    }
}

impl<'a> Print<'a> for Telescope {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        self.0.print(alloc)
    }
}

impl<'a> Print<'a> for Param {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Param { name, typ } = self;
        alloc.text(name).append(COLON).append(alloc.space()).append(typ.print(alloc))
    }
}

impl<'a> Print<'a> for TypApp {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypApp { name, subst } = self;
        alloc.typ(name).append(subst.print(alloc))
    }
}

impl<'a> Print<'a> for Exp {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Exp::Var { idx } => idx.print(alloc),
            Exp::TyCtor { name, subst } => alloc.typ(name).append(subst.print(alloc)),
            Exp::Ctor { name, subst } => alloc.ctor(name).append(subst.print(alloc)),
            Exp::Dtor { exp, name, subst } => {
                exp.print(alloc).append(DOT).append(alloc.dtor(name)).append(subst.print(alloc))
            }
            Exp::Ano { exp, typ } => {
                exp.print(alloc).parens().append(COLON).append(typ.print(alloc))
            }
            Exp::Type => alloc.typ(TYPE),
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
            alloc.intersperse(self.iter().map(|x| x.print(alloc)), sep).parens()
        }
    }
}
