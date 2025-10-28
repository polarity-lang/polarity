use lsp_types::{HoverContents, LanguageString, MarkedString, MarkupContent, MarkupKind};
use polarity_lang_miette_util::codespan::Span;
use rust_lapper::{Interval, Lapper};

use polarity_lang_ast::*;
use polarity_lang_printer::{Print, PrintCfg};
use url::Url;

use crate::result::MainResult;
use crate::Database;

use super::item::Item;
use super::lookup::{lookup_codef, lookup_ctor, lookup_decl, lookup_def, lookup_dtor, lookup_let};
use super::{Binder, Ctx};

/// Traverse the program and collect information for the LSP server.
#[allow(clippy::type_complexity)]
pub async fn collect_info(
    db: &mut Database,
    uri: &Url,
) -> MainResult<(Lapper<u32, HoverContents>, Lapper<u32, (Url, Span)>, Lapper<u32, Item>)> {
    let module = db.ast(uri).await?;
    let mut collector = InfoCollector::new(module.meta_vars.clone());

    for use_decl in module.use_decls.iter() {
        // Add hover info
        let content = MarkedString::String(format!("Import module `{}`", use_decl.path));
        let hover_content = HoverContents::Scalar(content);
        collector.add_hover(use_decl.span, hover_content);

        // Add goto info
        let dep_uri = db.resolve_module_name(&use_decl.path, uri)?;
        collector.add_goto(use_decl.span, (dep_uri, Span::default()));
    }

    for decl in module.decls.iter() {
        decl.collect_info(db, &mut collector)
    }

    let hover_lapper = Lapper::new(collector.hover_spans);
    let location_lapper = Lapper::new(collector.location_spans);
    let item_lapper = Lapper::new(collector.item_spans);
    Ok((hover_lapper, location_lapper, item_lapper))
}

fn string_to_language_string(s: String) -> MarkedString {
    MarkedString::LanguageString(LanguageString { language: "pol".to_owned(), value: s })
}

fn add_doc_comment(builder: &mut Vec<MarkedString>, doc: Option<Vec<String>>) {
    if let Some(doc) = doc {
        builder.push(MarkedString::String("---".to_owned()));
        for d in doc {
            builder.push(MarkedString::String(d))
        }
    }
}

fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

struct InfoCollector {
    meta_vars: HashMap<MetaVar, MetaVarState>,
    hover_spans: Vec<Interval<u32, HoverContents>>,
    location_spans: Vec<Interval<u32, (Url, Span)>>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl InfoCollector {
    fn new(meta_vars: HashMap<MetaVar, MetaVarState>) -> Self {
        InfoCollector { meta_vars, hover_spans: vec![], location_spans: vec![], item_spans: vec![] }
    }

    fn add_hover(&mut self, span: Span, hover: HoverContents) {
        let hover = Interval { start: span.start.0, stop: span.end.0, val: hover };
        self.hover_spans.push(hover)
    }

    fn add_goto(&mut self, span: Span, location: (Url, Span)) {
        let goto = Interval { start: span.start.0, stop: span.end.0, val: location };
        self.location_spans.push(goto)
    }

    fn add_item(&mut self, span: Span, item: Item) {
        let item = Interval { start: span.start.0, stop: span.end.0, val: item };
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
            Decl::Infix(infix) => infix.collect_info(db, collector),
            Decl::Note(note) => note.collect_info(db, collector),
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

            // Add hover info
            let params = typ.params.print_to_string(None);
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Data declaration: `{name}`")));
            add_doc_comment(&mut content, doc.clone().map(|doc| doc.docs));
            if !params.is_empty() {
                content.push(MarkedString::String("---".to_owned()).to_owned());
                content.push(MarkedString::String(format!("Parameters: `{params}`")));
            }
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
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

            // Add hover info
            let params = typ.params.print_to_string(None);
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Codata declaration: `{name}`")));
            add_doc_comment(&mut content, doc.clone().map(|doc| doc.docs));
            if !params.is_empty() {
                content.push(MarkedString::String("---".to_owned()).to_owned());
                content.push(MarkedString::String(format!("Parameters: `{params}`")));
            }
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
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
            // Add hover info
            let header = MarkedString::String("Definition".to_owned());
            let hover_contents = HoverContents::Scalar(header);
            collector.add_hover(*span, hover_contents);
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
            // Add hover_info
            let header = MarkedString::String("Codefinition".to_owned());
            let hover_contents = HoverContents::Scalar(header);
            collector.add_hover(*span, hover_contents);
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
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Constructor: `{name}`")));
            add_doc_comment(&mut content, doc);
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
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
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Destructor: `{name}`")));
            add_doc_comment(&mut content, doc);
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
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
            // Add hover info
            let header = MarkedString::String("Let-binding".to_owned());
            let hover_content = HoverContents::Scalar(header);
            collector.add_hover(*span, hover_content);
        }
        typ.collect_info(db, collector);
        body.collect_info(db, collector);
        params.collect_info(db, collector)
    }
}

impl CollectInfo for Infix {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Infix { span, .. } = self;
        if let Some(span) = span {
            // Add hover info
            let header = MarkedString::String("Infix declaration".to_owned());
            let hover_content = HoverContents::Scalar(header);
            collector.add_hover(*span, hover_content);
        }
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
            Exp::LocalLet(e) => e.collect_info(db, collector),
        }
    }
}

impl CollectInfo for Variable {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Variable { span, inferred_type, name, .. } = self;
        if let (Some(span), Some(typ)) = (span, inferred_type) {
            let typ = typ.print_to_string(None);
            let header = MarkedString::String(format!("Bound variable: `{}`", name.id));
            let typ = string_to_language_string(typ);
            let hover_content = HoverContents::Array(vec![header, typ]);
            collector.add_hover(*span, hover_content)
        }
    }
}

impl CollectInfo for TypCtor {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let TypCtor { span, args, name, is_bin_op: _ } = self;
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

            // Add hover info
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Type constructor: `{name}`")));
            add_doc_comment(&mut content, doc);
            let hover_contents = HoverContents::Array(content);
            collector.add_hover(*span, hover_contents);

            // Add goto info
            if let Some(goto) = definition_site {
                collector.add_goto(*span, goto)
            }
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
            // Add hover info
            let typ = typ.print_to_string(None);
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(match kind {
                CallKind::Constructor => {
                    MarkedString::String(format!("Constructor: `{}`", name.id))
                }
                CallKind::Codefinition => {
                    MarkedString::String(format!("Codefinition: `{}`", name.id))
                }
                CallKind::LetBound => {
                    MarkedString::String(format!("Let-bound definition: `{}`", name.id))
                }
            });
            add_doc_comment(&mut content, doc);
            content.push(MarkedString::String("---".to_owned()));
            content.push(string_to_language_string(typ));
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);

            // Add goto info
            if let Some(goto) = definition_site {
                collector.add_goto(*span, goto)
            }
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
            // Add hover info
            let typ = typ.print_to_string(None);
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(match kind {
                DotCallKind::Destructor => {
                    MarkedString::String(format!("Destructor: `{}`", name.id))
                }
                DotCallKind::Definition => {
                    MarkedString::String(format!("Definition: `{}`", name.id))
                }
            });
            add_doc_comment(&mut content, doc);
            content.push(MarkedString::String("---".to_owned()));
            content.push(string_to_language_string(typ));
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);

            // Add goto info
            if let Some(goto) = definition_site {
                collector.add_goto(*span, goto)
            }
        }
        exp.collect_info(db, collector);
        args.collect_info(db, collector)
    }
}

fn ctx_to_markdown(ctx: &Ctx, value: &mut String) {
    value.push_str("**Context**\n\n");
    value.push_str("| | |\n");
    value.push_str("|-|-|\n");
    for binder in ctx.bound.iter().rev().flatten() {
        match binder {
            Binder::Var { name, typ } => {
                value.push_str("| ");
                value.push_str(name);
                value.push_str(" | `");
                value.push_str(typ);
                value.push_str("` |\n");
            }
            Binder::Wildcard { .. } => continue,
        }
    }
}

fn goal_to_markdown(goal_type: &str, value: &mut String) {
    value.push_str("**Goal**\n\n");
    value.push_str("```\n");
    value.push_str(goal_type);
    value.push_str("\n```\n");
}

impl CollectInfo for Hole {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Hole { span, kind: _, metavar, inferred_type, inferred_ctx, args, solution: _ } = self;
        if let Some(span) = span {
            let metavar_state = collector
                .meta_vars
                .get(metavar)
                .unwrap_or_else(|| panic!("Metavar {metavar:?} not found"));

            let metavar_str = metavar_state.solution().map(|e| {
                e.print_to_string(Some(&PrintCfg { print_metavar_ids: true, ..Default::default() }))
            });
            let goal = inferred_type.print_to_string(None);
            let metavar = Some(format!("?{}", metavar.id));
            let ctx = inferred_ctx.clone().map(Into::into);
            let args: Vec<Vec<String>> = args
                .iter()
                .map(|subst| {
                    subst
                        .iter()
                        .map(|binder| {
                            format!("{}:={}", binder.name, binder.content.print_to_string(None))
                        })
                        .collect()
                })
                .collect();

            let hover_contents = if let Some(ctx) = ctx {
                let mut value = String::new();
                match metavar {
                    Some(mv) => value.push_str(&format!("Hole: `{mv}`\n\n")),
                    None => value.push_str("Hole: `?`\n\n"),
                }
                goal_to_markdown(&goal, &mut value);
                value.push_str("\n\n");
                ctx_to_markdown(&ctx, &mut value);
                value.push_str("\n\nArguments:\n\n");
                let args_str = args.iter().cloned().map(comma_separated).map(|s| format!("({s})"));
                let args_str = format!("({})", comma_separated(args_str));
                value.push_str(&args_str);
                if let Some(solution) = metavar_str {
                    value.push_str("\n\nSolution:\n\n");
                    value.push_str(&solution);
                }
                HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
            } else {
                HoverContents::Scalar(string_to_language_string(goal))
            };

            collector.add_hover(*span, hover_contents)
        }
    }
}

impl CollectInfo for TypeUniv {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let TypeUniv { span } = self;
        if let Some(span) = span {
            let content: Vec<MarkedString> = vec![
            MarkedString::String("Universe: `Type`".to_owned()),
            MarkedString::String("---".to_owned()),
            MarkedString::String(
                "The impredicative universe whose terms are types or the universe `Type` itself."
                    .to_owned(),
            ),
        ];
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
        }
    }
}

impl CollectInfo for Anno {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Anno { span, exp, typ, normalized_type } = self;
        if let (Some(span), Some(typ)) = (span, normalized_type) {
            let header = MarkedString::String("Annotated term".to_owned());
            let typ = typ.print_to_string(None);
            let typ = string_to_language_string(typ);
            let hover_contents = HoverContents::Array(vec![header, typ]);
            collector.add_hover(*span, hover_contents);
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
            let typ = typ.print_to_string(None);
            let header = MarkedString::String("Local match".to_owned());
            let typ = string_to_language_string(typ);
            let hover_contents = HoverContents::Array(vec![header, typ]);
            collector.add_hover(*span, hover_contents)
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
            let typ = typ.print_to_string(None);
            let header = MarkedString::String("Local comatch".to_owned());
            let typ = string_to_language_string(typ);
            let hover_content = HoverContents::Array(vec![header, typ]);
            collector.add_hover(*span, hover_content)
        }
        cases.collect_info(db, collector)
    }
}

impl CollectInfo for Pattern {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Pattern { span, name, is_copattern, .. } = self;
        if let Some(span) = span {
            let hover_contents = if *is_copattern {
                HoverContents::Array(vec![MarkedString::String(format!(
                    "Copattern: `{}`",
                    name.id
                ))])
            } else {
                HoverContents::Array(vec![MarkedString::String(format!("Pattern: `{}`", name.id))])
            };
            collector.add_hover(*span, hover_contents)
        }
    }
}

impl CollectInfo for Case {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let Case { body, span, pattern, .. } = self;
        if let Some(span) = span {
            let hover_contents =
                HoverContents::Array(vec![MarkedString::String("Clause".to_owned())]);
            collector.add_hover(*span, hover_contents)
        }
        pattern.collect_info(db, collector);
        body.collect_info(db, collector)
    }
}

impl CollectInfo for LocalLet {
    fn collect_info(&self, db: &Database, collector: &mut InfoCollector) {
        let LocalLet { span, name, typ, bound, body, inferred_type } = self;
        typ.collect_info(db, collector);
        bound.collect_info(db, collector);
        body.collect_info(db, collector);
        if let Some(typ) = inferred_type {
            // Add info
            let typ = typ.print_to_string(None);
            let header = MarkedString::String(format!("Local let-binding: `{name}`"));
            let typ = string_to_language_string(typ);
            let hover_content = HoverContents::Array(vec![header, typ]);
            collector.add_hover(*span, hover_content)
        }
    }
}

impl CollectInfo for Note {
    fn collect_info(&self, _db: &Database, collector: &mut InfoCollector) {
        let Note { name, span, doc, attr: _ } = self;
        if let Some(span) = span {
            // Add hover info
            let mut content: Vec<MarkedString> = Vec::new();
            content.push(MarkedString::String(format!("Note: `{name}`")));
            add_doc_comment(&mut content, doc.clone().map(|doc| doc.docs));
            let hover_content = HoverContents::Array(content);
            collector.add_hover(*span, hover_content);
        }
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
