use std::rc::Rc;

use codespan::Span;
use rust_lapper::{Interval, Lapper};

use printer::PrintToString;
use syntax::{common::HashMap, generic::*};

use super::data::{
    AnnoInfo, CallInfo, DotCallInfo, HoleInfo, Info, InfoContent, Item, TypeCtorInfo, TypeUnivInfo,
    VariableInfo,
};

/// Traverse the program and collect information for the LSP server.
pub fn collect_info(prg: &Prg) -> (Lapper<u32, Info>, Lapper<u32, Item>) {
    let mut collector = InfoCollector::default();
    let Prg { decls } = prg;
    let Decls { map, .. } = decls;
    for item in map.values() {
        item.collect_info(map, &mut collector)
    }

    //prg.collect_info(&mut c);

    let info_lapper = Lapper::new(collector.info_spans);
    let item_lapper = Lapper::new(collector.item_spans);
    (info_lapper, item_lapper)
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, Info>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn add_hover_content(&mut self, span: Span, content: InfoContent) {
        let info = Interval {
            start: span.start().into(),
            stop: span.end().into(),
            val: Info { span, content },
        };
        self.info_spans.push(info)
    }
}

/// Every syntax node which implements this trait can be traversed and
/// make source-code indexed information available for the LSP server.
trait CollectInfo {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, _collector: &mut InfoCollector) {}
}

// Generic implementations
//
//

impl<T: CollectInfo> CollectInfo for Rc<T> {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        (**self).collect_info(map, collector)
    }
}

impl<T: CollectInfo> CollectInfo for Option<T> {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        match self {
            None => (),
            Some(x) => x.collect_info(map, collector),
        }
    }
}

// Toplevel declarations
//
//

impl CollectInfo for Decl {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        match self {
            Decl::Data(data) => data.collect_info(map, collector),
            Decl::Codata(codata) => codata.collect_info(map, collector),
            Decl::Ctor(ctor) => ctor.collect_info(map, collector),
            Decl::Dtor(dtor) => dtor.collect_info(map, collector),
            Decl::Def(def) => def.collect_info(map, collector),
            Decl::Codef(codef) => codef.collect_info(map, collector),
            Decl::Let(lets) => lets.collect_info(map, collector),
        }
    }
}

impl CollectInfo for Data {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Data { name, span, .. } = self;
        if let Some(span) = span {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Data(name.clone()),
            };
            collector.item_spans.push(item)
        }
    }
}

impl CollectInfo for Codata {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Codata { name, span, .. } = self;
        if let Some(span) = span {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Codata(name.clone()),
            };
            collector.item_spans.push(item)
        }
    }
}

impl CollectInfo for Def {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Def { name, span, self_param, body, .. } = self;
        if let Some(span) = span {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Def { name: name.clone(), type_name: self_param.typ.name.clone() },
            };
            collector.item_spans.push(item);
        };

        body.collect_info(map, collector)
    }
}

impl CollectInfo for Codef {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Codef { name, span, typ, body, .. } = self;
        if let Some(span) = span {
            let item = Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: Item::Codef { name: name.clone(), type_name: typ.name.clone() },
            };
            collector.item_spans.push(item);
        }
        body.collect_info(map, collector)
    }
}

impl CollectInfo for Ctor {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, _collector: &mut InfoCollector) {}
}

impl CollectInfo for Dtor {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, _collector: &mut InfoCollector) {}
}

impl CollectInfo for Let {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Let { typ, body, .. } = self;
        typ.collect_info(map, collector);
        body.collect_info(map, collector)
    }
}

// Traversing expressions and collection information
//
//

impl CollectInfo for Exp {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        match self {
            Exp::Variable(e) => e.collect_info(map, collector),
            Exp::TypCtor(e) => e.collect_info(map, collector),
            Exp::Call(e) => e.collect_info(map, collector),
            Exp::DotCall(e) => e.collect_info(map, collector),
            Exp::Hole(e) => e.collect_info(map, collector),
            Exp::TypeUniv(e) => e.collect_info(map, collector),
            Exp::Anno(e) => e.collect_info(map, collector),
            Exp::LocalMatch(e) => e.collect_info(map, collector),
            Exp::LocalComatch(e) => e.collect_info(map, collector),
        }
    }
}

impl CollectInfo for Variable {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Variable { span, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content = VariableInfo { typ: typ.print_to_string(None) };
            collector.add_hover_content(*span, content.into())
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let TypCtor { span, args, name, .. } = self;
        if let Some(span) = span {
            let x = map.get(name);
            let target_span = match x {
                Some(Decl::Data(d)) => d.span,
                Some(Decl::Codata(d)) => d.span,
                _ => None,
            };
            let content = TypeCtorInfo { target_span };
            collector.add_hover_content(*span, content.into())
        }
        args.collect_info(map, collector)
    }
}

impl CollectInfo for Call {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Call { span, kind, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content = CallInfo { kind: *kind, typ: typ.print_to_string(None) };
            collector.add_hover_content(*span, content.into())
        }
        args.collect_info(map, collector)
    }
}

impl CollectInfo for DotCall {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let DotCall { span, kind, exp, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content = DotCallInfo { kind: *kind, typ: typ.print_to_string(None) };
            collector.add_hover_content(*span, content.into())
        }
        exp.collect_info(map, collector);
        args.collect_info(map, collector)
    }
}

impl CollectInfo for Hole {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Hole { span, inferred_type, inferred_ctx } = self;
        if let Some(span) = span {
            let content = HoleInfo {
                goal: inferred_type.print_to_string(None),
                ctx: inferred_ctx.clone().map(Into::into),
            };
            collector.add_hover_content(*span, content.into())
        }
    }
}

impl CollectInfo for TypeUniv {
    fn collect_info(&self, _map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let TypeUniv { span } = self;
        if let Some(span) = span {
            let content = TypeUnivInfo {};
            collector.add_hover_content(*span, content.into())
        }
    }
}

impl CollectInfo for Anno {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Anno { span, exp, typ, normalized_type } = self;
        if let (Some(span), Some(typ)) = (span, normalized_type) {
            let content = AnnoInfo { typ: typ.print_to_string(None) };
            collector.add_hover_content(*span, content.into())
        }
        exp.collect_info(map, collector);
        typ.collect_info(map, collector)
    }
}

impl CollectInfo for LocalMatch {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let LocalMatch { on_exp, ret_typ, body, .. } = self;
        on_exp.collect_info(map, collector);
        ret_typ.collect_info(map, collector);
        body.collect_info(map, collector)
    }
}

impl CollectInfo for LocalComatch {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let LocalComatch { body, .. } = self;
        body.collect_info(map, collector)
    }
}

impl CollectInfo for Match {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Match { cases, .. } = self;
        for case in cases.iter() {
            case.collect_info(map, collector)
        }
    }
}

impl CollectInfo for Case {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Case { body, .. } = self;
        body.collect_info(map, collector)
    }
}

impl CollectInfo for Args {
    fn collect_info(&self, map: &HashMap<Ident, Decl>, collector: &mut InfoCollector) {
        let Args { args } = self;
        for arg in args.iter() {
            arg.collect_info(map, collector)
        }
    }
}
