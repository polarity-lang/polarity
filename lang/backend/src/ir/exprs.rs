use url::Url;

use polarity_lang_ast::empty_braces;
use polarity_lang_printer::theme::ThemeExt;
use polarity_lang_printer::tokens::*;
use polarity_lang_printer::util::BracesExt;
use polarity_lang_printer::{Alloc, Builder, DocAllocator, Precedence, Print, PrintCfg};

use crate::ir::rename::{Rename, RenameCtx};

#[derive(Debug, Clone)]
pub enum Exp {
    Variable(Variable),
    CtorCall(Call),
    CodefCall(Call),
    LetCall(Call),
    ExternCall(Call),
    DtorCall(DotCall),
    DefCall(DotCall),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    LocalLet(LocalLet),
    Literal(Literal),
    Panic(Panic),
    /// Zero-Sized Term
    /// This term has no runtime effect and is generated as a placeholder whenever types cannot be erased by the current implementation.
    ZST,
}

impl Print for Exp {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Exp::Variable(v) => v.print_prec(cfg, alloc, prec),
            Exp::CtorCall(c) => c.print_prec(cfg, alloc, prec),
            Exp::CodefCall(c) => c.print_prec(cfg, alloc, prec),
            Exp::LetCall(c) => c.print_prec(cfg, alloc, prec),
            Exp::ExternCall(c) => c.print_prec(cfg, alloc, prec),
            Exp::DtorCall(d) => d.print_prec(cfg, alloc, prec),
            Exp::DefCall(d) => d.print_prec(cfg, alloc, prec),
            Exp::LocalMatch(m) => m.print_prec(cfg, alloc, prec),
            Exp::LocalComatch(m) => m.print_prec(cfg, alloc, prec),
            Exp::LocalLet(l) => l.print_prec(cfg, alloc, prec),
            Exp::Literal(l) => l.print_prec(cfg, alloc, prec),
            Exp::Panic(p) => p.print_prec(cfg, alloc, prec),
            Exp::ZST => alloc.keyword("<ZST>"),
        }
    }
}

impl Rename for Exp {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        match self {
            Exp::Variable(variable) => variable.rename(ctx),
            Exp::CtorCall(call) => call.rename(ctx),
            Exp::CodefCall(call) => call.rename(ctx),
            Exp::LetCall(call) => call.rename(ctx),
            Exp::ExternCall(call) => call.rename(ctx),
            Exp::DtorCall(dot_call) => dot_call.rename(ctx),
            Exp::DefCall(dot_call) => dot_call.rename(ctx),
            Exp::LocalMatch(local_match) => local_match.rename(ctx),
            Exp::LocalComatch(local_comatch) => local_comatch.rename(ctx),
            Exp::LocalLet(local_let) => local_let.rename(ctx),
            Exp::Literal(_) => (),
            Exp::Panic(_) => (),
            Exp::ZST => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl Print for Variable {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.text(&self.name)
    }
}

impl Rename for Variable {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: String,
    /// The URI of the module where `name` is defined.
    pub module_uri: Url,
    pub args: Vec<Exp>,
}

impl Print for Call {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Call { name, args, .. } = self;
        alloc.ctor(name).append(print_args(args, cfg, alloc))
    }
}

impl Rename for Call {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct DotCall {
    pub exp: Box<Exp>,
    /// The URI of the module where `name` is defined.
    pub module_uri: Url,
    pub name: String,
    pub args: Vec<Exp>,
}

impl Print for DotCall {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        // A series of destructors forms an aligned group
        let mut dtors_group = alloc.nil();

        // First DotCall
        dtors_group = alloc
            .text(DOT)
            .append(alloc.dtor(&self.name))
            .append(print_args(&self.args, cfg, alloc))
            .append(dtors_group);

        // Remaining DotCalls
        let mut dtor: &Exp = &self.exp;
        while let Exp::DtorCall(dot_call) | Exp::DefCall(dot_call) = &dtor {
            let DotCall { exp, name, args, .. } = dot_call;

            let psubst = if args.is_empty() { alloc.nil() } else { print_args(args, cfg, alloc) };
            dtors_group = alloc.line_().append(dtors_group);
            dtors_group =
                alloc.text(DOT).append(alloc.dtor(name)).append(psubst).append(dtors_group);
            dtor = exp;
        }
        dtor.print(cfg, alloc).append(dtors_group.align().group())
    }
}

impl Rename for DotCall {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LocalMatch {
    pub on_exp: Box<Exp>,
    pub cases: Vec<Case>,
}

impl Print for LocalMatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalMatch { on_exp, cases, .. } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(alloc.space())
            .append(print_cases(cases, cfg, alloc))
    }
}

impl Rename for LocalMatch {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    I64(i64),
    F64(f64),
    Char(char),
    String(String),
}

impl Print for Literal {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Literal::I64(val) => alloc.text(format!("{val}")),
            Literal::F64(val) => alloc.text(format!("{val:?}")),
            Literal::Char(val) => alloc.text(format!(r#"'{val}'"#)),
            Literal::String(val) => alloc.text(format!(r#""{val}""#)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalComatch {
    pub cases: Vec<Case>,
}

impl Print for LocalComatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalComatch { cases, .. } = self;
        alloc.keyword(COMATCH).append(alloc.space()).append(print_cases(cases, cfg, alloc))
    }
}

impl Rename for LocalComatch {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct LocalLet {
    pub name: String,
    pub bound: Box<Exp>,
    pub body: Box<Exp>,
}

impl Print for LocalLet {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalLet { name, bound, body } = self;
        alloc
            .keyword(LET)
            .append(alloc.space())
            .append(alloc.text(name))
            .append(alloc.space())
            .append(alloc.text(COLONEQ))
            .append(alloc.space())
            .append(bound.print(cfg, alloc))
            .append(alloc.keyword(SEMICOLON))
            .append(alloc.hardline())
            .append(body.print(cfg, alloc))
    }
}

impl Rename for LocalLet {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Panic {
    pub message: String,
}

impl Print for Panic {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let quoted = format!("\"{}\"", self.message.escape_default());
        alloc.keyword("panic!").append(alloc.text(quoted).parens())
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub pattern: Pattern,
    pub body: Box<Exp>,
}

impl Print for Case {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { pattern, body } = self;

        let body = alloc
            .text(FAT_ARROW)
            .append(alloc.line())
            .append(body.print(cfg, alloc))
            .nest(cfg.indent);

        pattern.print(cfg, alloc).append(alloc.space()).append(body).group()
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub is_copattern: bool,
    pub name: String,
    /// The URI of the module where `name` is defined.
    pub module_uri: Url,
    pub params: Vec<String>,
}

impl Print for Pattern {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Pattern { is_copattern, name, params, .. } = self;
        if *is_copattern {
            alloc.text(DOT).append(alloc.ctor(name)).append(print_params(params, alloc))
        } else {
            alloc.ctor(name).append(print_params(params, alloc))
        }
    }
}

impl Rename for Pattern {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

pub fn print_params<'a>(params: &'a [String], alloc: &'a Alloc<'a>) -> Builder<'a> {
    if params.is_empty() {
        return alloc.nil();
    }

    let mut doc = alloc.nil();
    let mut first = true;

    for param in params {
        if !first {
            doc = doc.append(COMMA).append(alloc.space());
        }
        doc = doc.append(alloc.text(param));
        first = false;
    }

    doc.align().parens().group()
}

pub fn print_cases<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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
                .append(
                    alloc
                        .intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep)
                        .append(alloc.text(COMMA).flat_alt(alloc.nil())),
                )
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_anno()
        }
    }
}

fn print_args<'a>(args: &'a [Exp], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    if args.is_empty() {
        return alloc.nil();
    }

    let mut doc = alloc.nil();
    let mut first = true;

    for arg in args {
        if !first {
            doc = doc.append(COMMA).append(alloc.line());
        }
        doc = doc.append(arg.print(cfg, alloc));
        first = false;
    }

    doc.align().parens().group()
}
