use codespan::Span;
use rust_lapper::{Interval, Lapper};

use ast::*;
use printer::{Print, PrintCfg};
use url::Url;

use crate::{
    CodataInfo, CodefInfo, CtorInfo, DataInfo, Database, DefInfo, DtorInfo, Error, LetInfo,
    LocalComatchInfo, LocalMatchInfo,
};

use super::data::{
    AnnoInfo, CallInfo, DotCallInfo, HoleInfo, Info, InfoContent, TypeCtorInfo, TypeUnivInfo,
    UseInfo, VariableInfo,
};
use super::item::Item;
use super::lookup::{lookup_codef, lookup_ctor, lookup_decl, lookup_def, lookup_dtor, lookup_let};

/// Traverse the program and collect information for the LSP server.
#[allow(clippy::type_complexity)]
pub async fn collect_info(
    db: &mut Database,
    uri: &Url,
) -> Result<(Lapper<u32, Info>, Lapper<u32, Item>), Error> {
    let module = db.ast(uri).await?;
    let mut collector = InfoCollector::new(module.meta_vars.clone());

    for use_decl in module.use_decls.iter() {
        let dep_uri = db.resolve_module_name(&use_decl.path, uri)?;
        let info = Info {
            span: use_decl.span,
            content: InfoContent::UseInfo(UseInfo { uri: dep_uri, path: use_decl.path.clone() }),
        };
        collector.info_spans.push(Interval {
            start: use_decl.span.start().into(),
            stop: use_decl.span.end().into(),
            val: info,
        })
    }

    for decl in module.decls.iter() {
        decl.collect_info(db, &mut collector)
    }

    let info_lapper = Lapper::new(collector.info_spans);
    let item_lapper = Lapper::new(collector.item_spans);
    Ok((info_lapper, item_lapper))
}

struct InfoCollector {
    meta_vars: HashMap<MetaVar, MetaVarState>,
    info_spans: Vec<Interval<u32, Info>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn new(meta_vars: HashMap<MetaVar, MetaVarState>) -> Self {
        InfoCollector { meta_vars, info_spans: vec![], item_spans: vec![] }
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
    fn collect_info(&self, _db: &Database, _collector: &mut InfoCollector);
}

// Generic implementations
//
//

impl<T: CollectInfo> CollectInfo for Box<T> {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        (**self).collect_info(db, collector)
    }
}

impl<T: CollectInfo> CollectInfo for Option<T> {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        match self {
            None => (),
            Some(x) => x.collect_info(db, collector),
        }
    }
}

impl<T: CollectInfo> CollectInfo for Vec<T> {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        for i in self {
            i.collect_info(db, collector)
        }
    }
}

// Traversing toplevel declarations
//
//

impl CollectInfo for Decl {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        match self {
            Decl::Data(data) => data.collect_info(db, collector),
            Decl::Codata(codata) => codata.collect_info(db, collector),
            Decl::Def(def) => def.collect_info(db, collector),
            Decl::Codef(codef) => codef.collect_info(db, collector),
            Decl::Let(lets) => lets.collect_info(db, collector),
        }
    }
}

impl CollectInfo for Data {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Data { name, span, doc, typ, ctors, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Data(name.clone().id);
            collector.add_item(*span, item);
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info =
                DataInfo { name: name.clone().id, doc, params: typ.params.print_to_string(None) };
            collector.add_info(*span, info)
        }

        for ctor in ctors {
            ctor.collect_info(db, collector)
        }

        typ.collect_info(db, collector)
    }
}

impl CollectInfo for Codata {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Codata { name, doc, typ, span, dtors, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codata(name.clone().id);
            collector.add_item(*span, item);
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info =
                CodataInfo { name: name.clone().id, doc, params: typ.params.print_to_string(None) };
            collector.add_info(*span, info);
        }

        for dtor in dtors {
            dtor.collect_info(db, collector)
        }

        typ.collect_info(db, collector)
    }
}

impl CollectInfo for Def {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Def { name, span, self_param, cases, params, ret_typ, .. } = self;
        if let Some(span) = span {
            // Add Item
            let item =
                Item::Def { name: name.clone().id, type_name: self_param.typ.name.clone().id };
            collector.add_item(*span, item);
            // Add Info
            let info = DefInfo {};
            collector.add_info(*span, info);
        };

        self_param.collect_info(db, collector);
        cases.collect_info(db, collector);
        params.collect_info(db, collector);
        ret_typ.collect_info(db, collector);
    }
}

impl CollectInfo for Codef {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Codef { name, span, typ, cases, params, .. } = self;
        if let Some(span) = span {
            // Add item
            let item = Item::Codef { name: name.clone().id, type_name: typ.name.clone().id };
            collector.add_item(*span, item);
            // Add info
            let info = CodefInfo {};
            collector.add_info(*span, info);
        }
        typ.collect_info(db, collector);
        cases.collect_info(db, collector);
        params.collect_info(db, collector)
    }
}

impl CollectInfo for Ctor {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Ctor { span, name, doc, typ, params } = self;
        if let Some(span) = span {
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info = CtorInfo { name: name.clone().id, doc };
            collector.add_info(*span, info);
        }
        params.collect_info(db, collector);
        typ.collect_info(db, collector);
    }
}

impl CollectInfo for Dtor {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Dtor { span, name, doc, self_param, params, ret_typ } = self;
        if let Some(span) = span {
            // Add info
            let doc = doc.clone().map(|doc| doc.docs);
            let info = DtorInfo { name: name.clone().id, doc };
            collector.add_info(*span, info);
        }
        self_param.collect_info(db, collector);
        params.collect_info(db, collector);
        ret_typ.collect_info(db, collector);
    }
}

impl CollectInfo for Let {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Let { span, typ, body, params, .. } = self;
        if let Some(span) = span {
            // Add info
            let info = LetInfo {};
            collector.add_info(*span, info);
        }
        typ.collect_info(db, collector);
        body.collect_info(db, collector);
        params.collect_info(db, collector)
    }
}

// Traversing expressions and collection information
//
//

impl CollectInfo for Exp {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        match self {
            Exp::Variable(e) => e.collect_info(db, collector),
            Exp::TypCtor(e) => e.collect_info(db, collector),
            Exp::Call(e) => e.collect_info(db, collector),
            Exp::DotCall(e) => e.collect_info(db, collector),
            Exp::Hole(e) => e.collect_info(db, collector),
            Exp::TypeUniv(e) => e.collect_info(db, collector),
            Exp::Anno(e) => e.collect_info(db, collector),
            Exp::LocalMatch(e) => e.collect_info(db, collector),
            Exp::LocalComatch(e) => e.collect_info(db, collector),
        }
    }
}

impl CollectInfo for Variable {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Variable { span, inferred_type, name, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let info = VariableInfo { typ: typ.print_to_string(None), name: name.clone().id };
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let TypCtor { span, args, name } = self;
        if let Some(span) = span {
            let decl = lookup_decl(db, name);
            let (definition_site, doc) = match decl {
                Some((uri, Decl::Data(d))) => {
                    let definition_site = d.span.map(|span| (uri.clone(), span));
                    let doc = d.doc.clone().map(|doc| doc.docs);
                    (definition_site, doc)
                }
                Some((uri, Decl::Codata(d))) => {
                    let definition_site = d.span.map(|span| (uri.clone(), span));
                    let doc = d.doc.clone().map(|doc| doc.docs);
                    (definition_site, doc)
                }
                _ => (None, None),
            };
            let info = TypeCtorInfo { name: name.clone().id, definition_site, doc };
            collector.add_info(*span, info)
        }
        args.collect_info(db, collector)
    }
}

impl CollectInfo for Call {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Call { span, kind, args, inferred_type, name } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let (definition_site, doc) = match kind {
                CallKind::Constructor => match lookup_ctor(db, name) {
                    Some((uri, ctor)) => {
                        let uri_span = ctor.span.map(|span| (uri.clone(), span));
                        let doc = ctor.doc.clone().map(|doc| doc.docs);
                        (uri_span, doc)
                    }
                    None => (None, None),
                },
                CallKind::Codefinition => match lookup_codef(db, name) {
                    Some((uri, codef)) => {
                        let uri_span = codef.span.map(|span| (uri.clone(), span));
                        let doc = codef.doc.clone().map(|doc| doc.docs);
                        (uri_span, doc)
                    }
                    None => (None, None),
                },
                CallKind::LetBound => match lookup_let(db, name) {
                    Some((uri, let_)) => {
                        let uri_span = let_.span.map(|span| (uri.clone(), span));
                        let doc = let_.doc.clone().map(|doc| doc.docs);
                        (uri_span, doc)
                    }
                    None => (None, None),
                },
            };

            let info = CallInfo {
                kind: *kind,
                doc,
                typ: typ.print_to_string(None),
                name: name.clone().id,
                definition_site,
            };
            collector.add_info(*span, info)
        }
        args.collect_info(db, collector)
    }
}

impl CollectInfo for DotCall {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let DotCall { span, kind, exp, args, inferred_type, name } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let (definition_site, doc) = match kind {
                DotCallKind::Destructor => match lookup_dtor(db, name) {
                    Some((uri, dtor)) => {
                        let uri_span = dtor.span.map(|span| (uri.clone(), span));
                        let doc = dtor.doc.clone().map(|doc| doc.docs);
                        (uri_span, doc)
                    }
                    None => (None, None),
                },
                DotCallKind::Definition => match lookup_def(db, name) {
                    Some((uri, def)) => {
                        let uri_span = def.span.map(|span| (uri.clone(), span));
                        let doc = def.doc.clone().map(|doc| doc.docs);
                        (uri_span, doc)
                    }
                    None => (None, None),
                },
            };
            let info = DotCallInfo {
                kind: *kind,
                doc,
                name: name.clone().id,
                typ: typ.print_to_string(None),
                definition_site,
            };
            collector.add_info(*span, info)
        }
        exp.collect_info(db, collector);
        args.collect_info(db, collector)
    }
}

impl CollectInfo for Hole {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Hole { span, kind: _, metavar, inferred_type, inferred_ctx, args, solution: _ } = self;
        if let Some(span) = span {
            let metavar_state = collector
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
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let TypeUniv { span } = self;
        if let Some(span) = span {
            let info = TypeUnivInfo {};
            collector.add_info(*span, info)
        }
    }
}

impl CollectInfo for Anno {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Anno { span, exp, typ, normalized_type } = self;
        if let (Some(span), Some(typ)) = (span, normalized_type) {
            let info = AnnoInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        exp.collect_info(db, collector);
        typ.collect_info(db, collector)
    }
}

impl CollectInfo for LocalMatch {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let LocalMatch { span, on_exp, ret_typ, cases, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            // Add info
            let info = LocalMatchInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        on_exp.collect_info(db, collector);
        ret_typ.collect_info(db, collector);
        cases.collect_info(db, collector)
    }
}

impl CollectInfo for LocalComatch {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let LocalComatch { span, cases, inferred_type, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            // Add info
            let info = LocalComatchInfo { typ: typ.print_to_string(None) };
            collector.add_info(*span, info)
        }
        cases.collect_info(db, collector)
    }
}

impl CollectInfo for Case {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Case { body, .. } = self;
        body.collect_info(db, collector)
    }
}

impl CollectInfo for Args {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Args { args } = self;
        for arg in args.iter() {
            arg.collect_info(db, collector)
        }
    }
}

impl CollectInfo for Arg {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.collect_info(db, collector),
            Arg::NamedArg { arg, .. } => arg.collect_info(db, collector),
            Arg::InsertedImplicitArg { hole, .. } => hole.collect_info(db, collector),
        }
    }
}

impl CollectInfo for Telescope {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        self.params.collect_info(db, collector)
    }
}

impl CollectInfo for Param {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Param { typ, .. } = self;
        typ.collect_info(db, collector)
    }
}

impl CollectInfo for SelfParam {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let SelfParam { typ, .. } = self;
        typ.collect_info(db, collector);
    }
}
