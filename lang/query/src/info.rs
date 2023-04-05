use std::rc::Rc;

use codespan::Span;
use printer::PrintToString;
use rust_lapper::{Interval, Lapper};

use syntax::common::*;
use syntax::generic::{Visit, Visitor};
use syntax::tst;

pub fn collect_info(prg: &tst::Prg) -> (Lapper<u32, Info>, Lapper<u32, Item>) {
    let mut c = InfoCollector::default();

    prg.visit(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub typ: String,
    pub span: Option<Span>,
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
        Info { typ: info.typ.forget().print_to_string(None), span: info.span }
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

impl Visitor<tst::TST> for InfoCollector {
    fn visit_info(&mut self, info: &tst::Info) {
        self.add_info(info);
    }

    fn visit_type_info(&mut self, info: &tst::TypeInfo) {
        self.add_typed_info(info);
    }

    fn visit_data(
        &mut self,
        info: &tst::Info,
        _doc: &Option<DocComment>,
        name: &Ident,
        _hidden: bool,
        _typ: &Rc<tst::TypAbs>,
        _ctors: &[Ident],
    ) {
        self.add_item_span(Item::Data(name.clone()), info.span.unwrap());
    }

    fn visit_codata(
        &mut self,
        info: &tst::Info,
        _doc: &Option<DocComment>,
        name: &Ident,
        _hidden: bool,
        _typ: &Rc<tst::TypAbs>,
        _dtors: &[Ident],
    ) {
        self.add_item_span(Item::Data(name.clone()), info.span.unwrap());
    }

    fn visit_def(
        &mut self,
        info: &tst::Info,
        _doc: &Option<DocComment>,
        name: &Ident,
        _hidden: bool,
        _params: &tst::Telescope,
        self_param: &tst::SelfParam,
        _ret_typ: &Rc<tst::Exp>,
        _body: &tst::Match,
    ) {
        self.add_item_span(
            Item::Def { name: name.clone(), type_name: self_param.typ.name.clone() },
            info.span.unwrap(),
        );
    }

    fn visit_codef(
        &mut self,
        info: &tst::Info,
        _doc: &Option<DocComment>,
        name: &Ident,
        _hidden: bool,
        _params: &tst::Telescope,
        typ: &tst::TypApp,
        _body: &tst::Comatch,
    ) {
        self.add_item_span(
            Item::Codef { name: name.clone(), type_name: typ.name.clone() },
            info.span.unwrap(),
        )
    }
}

impl InfoCollector {
    fn add_info(&mut self, _info: &tst::Info) {}

    fn add_typed_info(&mut self, info: &tst::TypeInfo) {
        if let Some(span) = info.span {
            self.info_spans.push(Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: info.clone().into(),
            });
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
