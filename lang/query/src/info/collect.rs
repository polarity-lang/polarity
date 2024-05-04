use std::rc::Rc;

use codespan::Span;
use rust_lapper::{Interval, Lapper};

use printer::PrintToString;
use syntax::generic::*;

use super::data::{
    AnnoInfo, CallInfo, DotCallInfo, HoleInfo, HoverInfo, HoverInfoContent, Item, TypeCtorInfo,
    TypeUnivInfo, VariableInfo,
};

/// Traverse the program and collect information for the LSP server.
pub fn collect_info(prg: &Prg) -> (Lapper<u32, HoverInfo>, Lapper<u32, Item>) {
    let mut c = InfoCollector::default();

    prg.collect_info(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper)
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, HoverInfo>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn add_hover_content(&mut self, span: Span, content: HoverInfoContent) {
        let info = Interval {
            start: span.start().into(),
            stop: span.end().into(),
            val: HoverInfo { span, content },
        };
        self.info_spans.push(info)
    }
}

/// Every syntax node which implements this trait can be traversed and
/// make source-code indexed information available for the LSP server.
trait CollectInfo {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

// Generic implementations
//
//

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

// Traversing a module and toplevel declarations
//
//

impl CollectInfo for Prg {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Prg { decls } = self;
        decls.collect_info(collector)
    }
}

impl CollectInfo for Decls {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Decls { map, .. } = self;
        for item in map.values() {
            item.collect_info(collector)
        }
    }
}

impl CollectInfo for Decl {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            Decl::Data(data) => data.collect_info(collector),
            Decl::Codata(codata) => codata.collect_info(collector),
            Decl::Ctor(ctor) => ctor.collect_info(collector),
            Decl::Dtor(dtor) => dtor.collect_info(collector),
            Decl::Def(def) => def.collect_info(collector),
            Decl::Codef(codef) => codef.collect_info(collector),
            Decl::Let(lets) => lets.collect_info(collector),
        }
    }
}

impl CollectInfo for Data {
    fn collect_info(&self, collector: &mut InfoCollector) {
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
    fn collect_info(&self, collector: &mut InfoCollector) {
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
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Def { name, span, self_param, body, .. } = self;
        if let Some(span) = span {
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

impl CollectInfo for Codef {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Codef { name, span, typ, body, .. } = self;
        if let Some(span) = span {
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

impl CollectInfo for Ctor {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

impl CollectInfo for Dtor {
    fn collect_info(&self, _collector: &mut InfoCollector) {}
}

impl CollectInfo for Let {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Let { typ, body, .. } = self;
        typ.collect_info(collector);
        body.collect_info(collector)
    }
}

// Traversing expressions and collection information
//
//

impl CollectInfo for Exp {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            Exp::Variable(e) => e.collect_info(collector),
            Exp::TypCtor(e) => e.collect_info(collector),
            Exp::Call(e) => e.collect_info(collector),
            Exp::DotCall(e) => e.collect_info(collector),
            Exp::Hole(e) => e.collect_info(collector),
            Exp::TypeUniv(e) => e.collect_info(collector),
            Exp::Anno(e) => e.collect_info(collector),
            Exp::LocalMatch(e) => e.collect_info(collector),
            Exp::LocalComatch(e) => e.collect_info(collector),
        }
    }
}

impl CollectInfo for Variable {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Variable { span, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content =
                HoverInfoContent::VariableInfo(VariableInfo { typ: typ.print_to_string(None) });
            collector.add_hover_content(*span, content)
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let TypCtor { span, args, .. } = self;
        if let Some(span) = span {
            let content = HoverInfoContent::TypeCtorInfo(TypeCtorInfo {});
            collector.add_hover_content(*span, content)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for Call {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Call { span, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content = HoverInfoContent::CallInfo(CallInfo { typ: typ.print_to_string(None) });
            collector.add_hover_content(*span, content)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for DotCall {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let DotCall { span, exp, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let content =
                HoverInfoContent::DotCallInfo(DotCallInfo { typ: typ.print_to_string(None) });
            collector.add_hover_content(*span, content)
        }
        exp.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for Hole {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Hole { span, inferred_type, inferred_ctx } = self;
        if let Some(span) = span {
            let content = HoverInfoContent::HoleInfo(HoleInfo {
                goal: inferred_type.print_to_string(None),
                ctx: inferred_ctx.clone().map(Into::into),
            });
            collector.add_hover_content(*span, content)
        }
    }
}

impl CollectInfo for TypeUniv {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let TypeUniv { span } = self;
        if let Some(span) = span {
            let content = HoverInfoContent::TypeUnivInfo(TypeUnivInfo {});
            collector.add_hover_content(*span, content)
        }
    }
}

impl CollectInfo for Anno {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Anno { span, exp, typ, normalized_type } = self;
        if let (Some(span), Some(typ)) = (span, normalized_type) {
            let content = HoverInfoContent::AnnoInfo(AnnoInfo { typ: typ.print_to_string(None) });
            collector.add_hover_content(*span, content)
        }
        exp.collect_info(collector);
        typ.collect_info(collector)
    }
}

impl CollectInfo for LocalMatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let LocalMatch { on_exp, ret_typ, body, .. } = self;
        on_exp.collect_info(collector);
        ret_typ.collect_info(collector);
        body.collect_info(collector)
    }
}

impl CollectInfo for LocalComatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let LocalComatch { body, .. } = self;
        body.collect_info(collector)
    }
}

impl CollectInfo for Match {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Match { cases, .. } = self;
        for case in cases.iter() {
            case.collect_info(collector)
        }
    }
}

impl CollectInfo for Case {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Case { body, .. } = self;
        body.collect_info(collector)
    }
}

impl CollectInfo for Args {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Args { args } = self;
        for arg in args.iter() {
            arg.collect_info(collector)
        }
    }
}
