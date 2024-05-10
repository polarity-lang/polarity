use std::rc::Rc;

use codespan::Span;
use rust_lapper::{Interval, Lapper};

use printer::PrintToString;
use syntax::ast::*;

use crate::{CodataInfo, CodefInfo, DataInfo, DefInfo, LetInfo, LocalComatchInfo, LocalMatchInfo};

use super::data::{
    AnnoInfo, CallInfo, DotCallInfo, HoleInfo, Info, InfoContent, TypeCtorInfo, TypeUnivInfo,
    VariableInfo,
};
use super::item::Item;

/// Traverse the program and collect information for the LSP server.
pub fn collect_info(prg: &Module) -> (Lapper<u32, Info>, Lapper<u32, Item>) {
    let mut c = InfoCollector::default();

    prg.collect_info(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper)
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, Info>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn add_info<T: Into<InfoContent>>(&mut self, span: Span, info: T) {
        let info = Interval {
            start: span.start().into(),
            stop: span.end().into(),
            val: Info { span, content: info.into() },
        };
        self.info_spans.push(info)
    }

    fn add_item(&mut self, span: Span, item: Item) {
        let item = Interval { start: span.start().into(), stop: span.end().into(), val: item };
        self.item_spans.push(item)
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

impl CollectInfo for Module {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Module { map, .. } = self;
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
            // Add item
            let item = Item::Data(name.clone());
            collector.add_item(*span, item);
            // Add info
            let info = DataInfo {};
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for Codata {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Codata { name, span, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codata(name.clone());
            collector.add_item(*span, item);
            // Add info
            let info = CodataInfo {};
            collector.add_info(*span, info);
        }
    }
}

impl CollectInfo for Def {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Def { name, span, self_param, body, .. } = self;
        if let Some(span) = span {
            // Add Item
            let item = Item::Def { name: name.clone(), type_name: self_param.typ.name.clone() };
            collector.add_item(*span, item);
            // Add Info
            let info = DefInfo {};
            collector.add_info(*span, info);
        };

        body.collect_info(collector)
    }
}

impl CollectInfo for Codef {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Codef { name, span, typ, body, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codef { name: name.clone(), type_name: typ.name.clone() };
            collector.add_item(*span, item);
            // Add info
            let info = CodefInfo {};
            collector.add_info(*span, info);
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
        let Let { span, typ, body, .. } = self;
        if let Some(span) = span {
            // Add info
            let info = LetInfo {};
            collector.add_info(*span, info);
        }
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
            let info = VariableInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let TypCtor { span, args, .. } = self;
        if let Some(span) = span {
            let info = TypeCtorInfo {};
            collector.add_info(*span, info)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for Call {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Call { span, kind, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let info = CallInfo { kind: *kind, typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for DotCall {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let DotCall { span, kind, exp, args, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let info = DotCallInfo { kind: *kind, typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        exp.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for Hole {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Hole { span, inferred_type, inferred_ctx } = self;
        if let Some(span) = span {
            let info = HoleInfo {
                goal: inferred_type.print_to_string(None),
                ctx: inferred_ctx.clone().map(Into::into),
            };
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for TypeUniv {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let TypeUniv { span } = self;
        if let Some(span) = span {
            let info = TypeUnivInfo {};
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for Anno {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Anno { span, exp, typ, normalized_type } = self;
        if let (Some(span), Some(typ)) = (span, normalized_type) {
            let info = AnnoInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        exp.collect_info(collector);
        typ.collect_info(collector)
    }
}

impl CollectInfo for LocalMatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let LocalMatch { span, on_exp, ret_typ, body, .. } = self;
        if let Some(span) = span {
            // Add info
            let info = LocalMatchInfo {};
            collector.add_info(*span, info)
        }
        on_exp.collect_info(collector);
        ret_typ.collect_info(collector);
        body.collect_info(collector)
    }
}

impl CollectInfo for LocalComatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let LocalComatch { span, body, .. } = self;
        if let Some(span) = span {
            // Add info
            let info = LocalComatchInfo {};
            collector.add_info(*span, info)
        }
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
