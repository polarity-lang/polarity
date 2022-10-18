use std::rc::Rc;

use codespan::Span;
use printer::PrintToString;
use rust_lapper::{Interval, Lapper};

use syntax::common::*;
use syntax::elab;
use syntax::generic::{Visit, Visitor};

pub fn collect_info(prg: &elab::Prg) -> (Lapper<u32, usize>, Lapper<u32, Item>, Vec<Info>) {
    let mut c = InfoCollector::default();

    prg.visit(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper, c.infos.into_iter().map(Into::into).collect())
}

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: String,
    pub span: Option<Span>,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Item {
    Data(String),
    Codata(String),
    Impl(String),
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, usize>>,
    infos: Vec<elab::TypedInfo>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl From<elab::TypedInfo> for Info {
    fn from(info: elab::TypedInfo) -> Self {
        Info { typ: info.typ.print_to_string(), span: info.span }
    }
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Data(name) => name,
            Item::Codata(name) => name,
            Item::Impl(name) => name,
        }
    }
}

impl Visitor<elab::Elab> for InfoCollector {
    fn visit_info(&mut self, info: &elab::Info) {
        self.add_info(info);
    }

    fn visit_type_info(&mut self, info: &elab::TypedInfo) {
        self.add_typed_info(info);
    }

    fn visit_data(
        &mut self,
        info: &elab::Info,
        name: &Ident,
        _typ: &Rc<elab::TypAbs>,
        _ctors: &[Ident],
        _impl_block: &Option<elab::Impl>,
    ) {
        self.add_item_span(Item::Data(name.clone()), info.span.unwrap());
    }

    fn visit_codata(
        &mut self,
        info: &elab::Info,
        name: &Ident,
        _typ: &Rc<elab::TypAbs>,
        _dtors: &[Ident],
        _impl_block: &Option<elab::Impl>,
    ) {
        self.add_item_span(Item::Data(name.clone()), info.span.unwrap());
    }

    fn visit_impl(&mut self, info: &elab::Info, name: &Ident, _defs: &[Ident]) {
        self.add_item_span(Item::Impl(name.clone()), info.span.unwrap());
    }
}

impl InfoCollector {
    fn add_info(&mut self, _info: &elab::Info) {}

    fn add_typed_info(&mut self, info: &elab::TypedInfo) {
        if let Some(span) = info.span {
            let idx = self.infos.len();
            self.info_spans.push(Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: idx,
            });
            self.infos.push(info.clone());
        }
    }

    fn add_item_span(&mut self, item: Item, span: Span) {
        self.item_spans.push(Interval {
            start: span.start().into(),
            stop: span.end().into(),
            val: item,
        })
    }
}
