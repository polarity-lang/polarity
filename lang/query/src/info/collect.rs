use std::sync::Arc;

use codespan::Span;
use rust_lapper::{Interval, Lapper};

use ast::*;
use printer::{Print, PrintCfg};

use crate::{
    CodataInfo, CodefInfo, CtorInfo, DataInfo, DefInfo, DtorInfo, LetInfo, LocalComatchInfo,
    LocalMatchInfo,
};

use super::data::{
    AnnoInfo, CallInfo, DotCallInfo, HoleInfo, Info, InfoContent, TypeCtorInfo, TypeUnivInfo,
    VariableInfo,
};
use super::item::Item;

/// Traverse the program and collect information for the LSP server.
pub fn collect_info(module: Arc<Module>) -> (Lapper<u32, Info>, Lapper<u32, Item>) {
    let mut collector = InfoCollector::new(module.clone());

    for decl in module.decls.iter() {
        decl.collect_info(&mut collector)
    }

    let info_lapper = Lapper::new(collector.info_spans);
    let item_lapper = Lapper::new(collector.item_spans);
    (info_lapper, item_lapper)
}

struct InfoCollector {
    module: Arc<Module>,
    info_spans: Vec<Interval<u32, Info>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn new(module: Arc<Module>) -> Self {
        InfoCollector { module, info_spans: vec![], item_spans: vec![] }
    }

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

impl<T: CollectInfo> CollectInfo for Box<T> {
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

impl<T: CollectInfo> CollectInfo for Vec<T> {
    fn collect_info(&self, collector: &mut InfoCollector) {
        for i in self {
            i.collect_info(collector)
        }
    }
}

// Traversing toplevel declarations
//
//

impl CollectInfo for Decl {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            Decl::Data(data) => data.collect_info(collector),
            Decl::Codata(codata) => codata.collect_info(collector),
            Decl::Def(def) => def.collect_info(collector),
            Decl::Codef(codef) => codef.collect_info(collector),
            Decl::Let(lets) => lets.collect_info(collector),
        }
    }
}

impl CollectInfo for Data {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Data { name, span, doc, typ, ctors, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Data(name.clone());
            collector.add_item(*span, item);
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info =
                DataInfo { name: name.clone(), doc, params: typ.params.print_to_string(None) };
            collector.add_info(*span, info)
        }

        for ctor in ctors {
            ctor.collect_info(collector)
        }

        typ.collect_info(collector)
    }
}

impl CollectInfo for Codata {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Codata { name, doc, typ, span, dtors, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codata(name.clone());
            collector.add_item(*span, item);
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info =
                CodataInfo { name: name.clone(), doc, params: typ.params.print_to_string(None) };
            collector.add_info(*span, info);
        }

        for dtor in dtors {
            dtor.collect_info(collector)
        }

        typ.collect_info(collector)
    }
}

impl CollectInfo for Def {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Def { name, span, self_param, cases, params, ret_typ, .. } = self;
        if let Some(span) = span {
            // Add Item
            let item = Item::Def { name: name.clone(), type_name: self_param.typ.name.clone() };
            collector.add_item(*span, item);
            // Add Info
            let info = DefInfo {};
            collector.add_info(*span, info);
        };

        self_param.collect_info(collector);
        cases.collect_info(collector);
        params.collect_info(collector);
        ret_typ.collect_info(collector);
    }
}

impl CollectInfo for Codef {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Codef { name, span, typ, cases, params, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codef { name: name.clone(), type_name: typ.name.clone() };
            collector.add_item(*span, item);
            // Add info
            let info = CodefInfo {};
            collector.add_info(*span, info);
        }
        typ.collect_info(collector);
        cases.collect_info(collector);
        params.collect_info(collector)
    }
}

impl CollectInfo for Ctor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Ctor { span, name, doc, typ, params } = self;
        if let Some(span) = span {
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info = CtorInfo { name: name.clone(), doc };
            collector.add_info(*span, info);
        }
        params.collect_info(collector);
        typ.collect_info(collector);
    }
}

impl CollectInfo for Dtor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Dtor { span, name, doc, self_param, params, ret_typ } = self;
        if let Some(span) = span {
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info = DtorInfo { name: name.clone(), doc };
            collector.add_info(*span, info);
        }
        self_param.collect_info(collector);
        params.collect_info(collector);
        ret_typ.collect_info(collector);
    }
}

impl CollectInfo for Let {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Let { span, typ, body, params, .. } = self;
        if let Some(span) = span {
            // Add info
            let info = LetInfo {};
            collector.add_info(*span, info);
        }
        typ.collect_info(collector);
        body.collect_info(collector);
        params.collect_info(collector)
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
        let Variable { span, inferred_type, name, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let info = VariableInfo { typ: typ.print_to_string(None), name: name.clone() };
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let TypCtor { span, args, name } = self;
        if let Some(span) = span {
            let decl = collector.module.lookup_decl(name);
            let definition_site = match decl {
                Some(Decl::Data(d)) => d.span.map(|span| (collector.module.uri.clone(), span)),
                Some(Decl::Codata(d)) => d.span.map(|span| (collector.module.uri.clone(), span)),
                _ => None,
            };
            let doc = match decl {
                Some(Decl::Data(d)) => d.doc.clone().map(|doc| doc.docs),
                Some(Decl::Codata(d)) => d.doc.clone().map(|doc| doc.docs),
                _ => None,
            };
            let info = TypeCtorInfo { name: name.clone(), definition_site, doc };
            collector.add_info(*span, info)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for Call {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Call { span, kind, args, inferred_type, name } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let (definition_site, doc) = match kind {
                CallKind::Constructor => {
                    let ctor = collector.module.lookup_ctor(name);

                    let uri_span = ctor.and_then(|ctor| {
                        ctor.span.map(|span| (collector.module.uri.clone(), span))
                    });

                    let doc = ctor.and_then(|ctor| ctor.doc.clone().map(|doc| doc.docs));

                    (uri_span, doc)
                }
                CallKind::Codefinition => {
                    let codef = collector.module.lookup_codef(name);

                    let uri_span = codef.and_then(|codef| {
                        codef.span.map(|span| (collector.module.uri.clone(), span))
                    });

                    let doc = codef.and_then(|codef| codef.doc.clone().map(|doc| doc.docs));

                    (uri_span, doc)
                }
                CallKind::LetBound => {
                    let let_ = collector.module.lookup_let(name);

                    let uri_span = let_.and_then(|let_| {
                        let_.span.map(|span| (collector.module.uri.clone(), span))
                    });

                    let doc = let_.and_then(|let_| let_.doc.clone().map(|doc| doc.docs));

                    (uri_span, doc)
                }
            };

            let info = CallInfo {
                kind: *kind,
                doc,
                typ: typ.print_to_string(None),
                name: name.clone(),
                definition_site,
            };
            collector.add_info(*span, info)
        }
        args.collect_info(collector)
    }
}

impl CollectInfo for DotCall {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let DotCall { span, kind, exp, args, inferred_type, name } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let (definition_site, doc) = match kind {
                DotCallKind::Destructor => {
                    let dtor = collector.module.lookup_dtor(name);

                    let uri_span = dtor.and_then(|dtor| {
                        dtor.span.map(|span| (collector.module.uri.clone(), span))
                    });

                    let doc = dtor.and_then(|dtor| dtor.doc.clone().map(|doc| doc.docs));

                    (uri_span, doc)
                }
                DotCallKind::Definition => {
                    let def = collector.module.lookup_def(name);

                    let uri_span = def
                        .and_then(|def| def.span.map(|span| (collector.module.uri.clone(), span)));

                    let doc = def.and_then(|def| def.doc.clone().map(|doc| doc.docs));

                    (uri_span, doc)
                }
            };
            let info = DotCallInfo {
                kind: *kind,
                doc,
                name: name.clone(),
                typ: typ.print_to_string(None),
                definition_site,
            };
            collector.add_info(*span, info)
        }
        exp.collect_info(collector);
        args.collect_info(collector)
    }
}

impl CollectInfo for Hole {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Hole { span, metavar, inferred_type, inferred_ctx, args } = self;
        if let Some(span) = span {
            let metavar_state = collector
                .module
                .meta_vars
                .get(metavar)
                .unwrap_or_else(|| panic!("Metavar {:?} not found", metavar));

            let metavar_str = metavar_state.solution().map(|e| {
                e.print_to_string(Some(&PrintCfg { print_metavar_ids: true, ..Default::default() }))
            });

            let info = HoleInfo {
                goal: inferred_type.print_to_string(None),
                metavar: Some(format!("?{}", metavar.id)),
                ctx: inferred_ctx.clone().map(Into::into),
                args: args
                    .iter()
                    .map(|subst| subst.iter().map(|exp| exp.print_to_string(None)).collect())
                    .collect(),
                metavar_state: metavar_str,
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
        let LocalMatch { span, on_exp, ret_typ, cases, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            // Add info
            let info = LocalMatchInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        on_exp.collect_info(collector);
        ret_typ.collect_info(collector);
        cases.collect_info(collector)
    }
}

impl CollectInfo for LocalComatch {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let LocalComatch { span, cases, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            // Add info
            let info = LocalComatchInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        cases.collect_info(collector)
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

impl CollectInfo for Arg {
    fn collect_info(&self, collector: &mut InfoCollector) {
        match self {
            Arg::UnnamedArg(exp) => exp.collect_info(collector),
            Arg::NamedArg(_, exp) => exp.collect_info(collector),
            Arg::InsertedImplicitArg(hole) => hole.collect_info(collector),
        }
    }
}

impl CollectInfo for Telescope {
    fn collect_info(&self, collector: &mut InfoCollector) {
        self.params.collect_info(collector)
    }
}

impl CollectInfo for Param {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let Param { typ, .. } = self;
        typ.collect_info(collector)
    }
}

impl CollectInfo for SelfParam {
    fn collect_info(&self, collector: &mut InfoCollector) {
        let SelfParam { typ, .. } = self;
        typ.collect_info(collector);
    }
}
