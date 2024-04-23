use std::rc::Rc;

use codespan::Span;
use printer::PrintToString;
use rust_lapper::{Interval, Lapper};

use syntax::ctx::values::{Binder as TypeCtxBinder, TypeCtx};
use syntax::tst::{self};

pub fn collect_info(prg: &tst::Prg) -> (Lapper<u32, Info>, Lapper<u32, Item>) {
    let mut c = InfoCollector::default();

    prg.collect_info(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub typ: String,
    pub span: Option<Span>,
    pub ctx: Option<Ctx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ctx {
    pub bound: Vec<Vec<Binder>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binder {
    pub name: String,
    pub typ: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Item {
    Data(String),
    Codata(String),
    Def { name: String, type_name: String },
    Codef { name: String, type_name: String },
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, Info>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl From<tst::TypeInfo> for Info {
    fn from(info: tst::TypeInfo) -> Self {
        Info { typ: info.typ.print_to_string(None), ctx: info.ctx.map(Into::into), span: info.span }
    }
}

impl From<TypeCtx> for Ctx {
    fn from(ctx: TypeCtx) -> Self {
        let bound =
            ctx.bound.into_iter().map(|tel| tel.into_iter().map(Into::into).collect()).collect();
        Ctx { bound }
    }
}

impl From<TypeCtxBinder> for Binder {
    fn from(binder: TypeCtxBinder) -> Self {
        Binder { name: binder.name, typ: binder.typ.print_to_string(None) }
    }
}

impl Item {
    pub fn type_name(&self) -> &str {
        match self {
            Item::Data(name) => name,
            Item::Codata(name) => name,
            Item::Def { type_name, .. } => type_name,
            Item::Codef { type_name, .. } => type_name,
        }
    }
}

trait CollectInfo {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

impl CollectInfo for tst::Prg {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Prg { decls } = self;
        decls.collect_info(collector)
    }
}

impl CollectInfo for tst::Decls {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Decls { map, .. } = self;
        for item in map.values() {
            item.collect_info(collector)
        }
    }
}

impl CollectInfo for tst::Decl {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            tst::Decl::Data(data) => data.collect_info(collector),
            tst::Decl::Codata(codata) => codata.collect_info(collector),
            tst::Decl::Ctor(ctor) => ctor.collect_info(collector),
            tst::Decl::Dtor(dtor) => dtor.collect_info(collector),
            tst::Decl::Def(def) => def.collect_info(collector),
            tst::Decl::Codef(codef) => codef.collect_info(collector),
            tst::Decl::Let(lets) => lets.collect_info(collector),
        }
    }
}

impl CollectInfo for tst::Data {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Data { name, info, .. } = self;
        if let Some(span) = info {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Data(name.clone()),
            };
            collector.item_spans.push(item)
        }
    }
}

impl CollectInfo for tst::Codata {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Codata { name, info, .. } = self;
        if let Some(span) = info {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Codata(name.clone()),
            };
            collector.item_spans.push(item)
        }
    }
}

impl CollectInfo for tst::Def {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Def { name, info, self_param, body, .. } = self;
        if let Some(span) = info {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Def { name: name.clone(), type_name: self_param.typ.name.clone() },
            };
            collector.item_spans.push(item);
        };

        body.collect_info(collector)
    }
}

impl CollectInfo for tst::Codef {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Codef { name, info, typ, body, .. } = self;
        if let Some(span) = info {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Codef { name: name.clone(), type_name: typ.name.clone() },
            };
            collector.item_spans.push(item);
        }
        body.collect_info(collector)
    }
}

impl CollectInfo for tst::Ctor {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

impl CollectInfo for tst::Dtor {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

impl CollectInfo for tst::Let {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Let { typ, body, .. } = self;
        typ.collect_info(collector);
        body.collect_info(collector)
    }
}

impl CollectInfo for tst::Match {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Match { cases, .. } = self;
        for case in cases.iter() {
            case.collect_info(collector)
        }
    }
}

impl CollectInfo for tst::Case {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Case { body, .. } = self;
        body.collect_info(collector)
    }
}

impl CollectInfo for tst::Args {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Args { args } = self;
        for arg in args.iter() {
            arg.collect_info(collector)
        }
    }
}

impl CollectInfo for tst::Exp {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            tst::Exp::Variable(e) => e.collect_info(collector),
            tst::Exp::TypCtor(e) => e.collect_info(collector),
            tst::Exp::Call(e) => e.collect_info(collector),
            tst::Exp::DotCall(e) => e.collect_info(collector),
            tst::Exp::Hole(e) => e.collect_info(collector),
            tst::Exp::Type(e) => e.collect_info(collector),
            tst::Exp::Anno(e) => e.collect_info(collector),
            tst::Exp::LocalMatch(e) => e.collect_info(collector),
            tst::Exp::LocalComatch(e) => e.collect_info(collector),
        }
    }
}

impl CollectInfo for tst::Variable {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Variable { info, .. } = self;
        info.collect_info(collector)
    }
}

impl CollectInfo for tst::TypCtor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::TypCtor { info, args, .. } = self;
        info.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for tst::Call {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Call { info, args, .. } = self;
        info.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for tst::DotCall {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::DotCall { info, exp, args, .. } = self;
        info.collect_info(collector);
        exp.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for tst::Hole {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Hole { info } = self;
        info.collect_info(collector)
    }
}

impl CollectInfo for tst::Type {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Type { info } = self;
        info.collect_info(collector)
    }
}

impl CollectInfo for tst::Anno {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::Anno { info, exp, typ } = self;
        info.collect_info(collector);
        exp.collect_info(collector);
        typ.collect_info(collector)
    }
}

impl CollectInfo for tst::LocalMatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::LocalMatch { on_exp, ret_typ, body, .. } = self;
        on_exp.collect_info(collector);
        ret_typ.as_exp().collect_info(collector);
        body.collect_info(collector)
    }
}

impl CollectInfo for tst::LocalComatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::LocalComatch { body, .. } = self;
        body.collect_info(collector)
    }
}

impl CollectInfo for tst::TypeInfo {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let tst::TypeInfo { typ, span, ctx } = self;
        if let Some(span) = span {
            let info = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Info {
                    typ: typ.print_to_string(None),
                    span: Some(*span),
                    ctx: ctx.clone().map(Into::into),
                },
            };
            collector.info_spans.push(info)
        }
    }
}
impl<T: CollectInfo> CollectInfo for Rc<T> {
    fn collect_info(&self, collector: &mut InfoCollector) {
        (**self).collect_info(collector)
    }
}

impl<T: CollectInfo> CollectInfo for Option<T> {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            None => (),
            Some(x) => x.collect_info(collector),
        }
    }
}
