use codespan::{ByteIndex, FileId, Files, Location, Span};
use printer::{PrintToString, PrintToStringInCtx};
use renaming::Rename;
use rust_lapper::{Interval, Lapper};

use data::HashMap;
use syntax::ast;
use syntax::elab;
use syntax::forget::Forget;

mod info;
mod result;

use info::{Collect, Collector, Item};
use result::Error;

pub struct Index {
    id_by_name: HashMap<String, FileId>,
    files: Files<String>,
    info_index_by_id: HashMap<FileId, Lapper<u32, usize>>,
    item_index_by_id: HashMap<FileId, Lapper<u32, Item>>,
    infos_by_id: HashMap<FileId, Vec<Info>>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: String,
    pub span: Option<Span>,
}

pub struct Xfunc {
    pub title: String,
    pub edits: Vec<Edit>,
}

pub struct Edit {
    pub span: Span,
    pub text: String,
}

impl Index {
    pub fn empty() -> Self {
        Self {
            id_by_name: HashMap::default(),
            files: Files::new(),
            info_index_by_id: HashMap::default(),
            item_index_by_id: HashMap::default(),
            infos_by_id: HashMap::default(),
        }
    }

    pub fn add(&mut self, name: &str, source: String) -> Result<(), String> {
        let id = self.files.add(name, source);
        self.id_by_name.insert(name.to_owned(), id);
        self.reload(id)
    }

    pub fn update(&mut self, name: &str, source: String) -> Result<(), String> {
        let id = self.id_by_name[name];
        self.files.update(id, source);
        self.reload(id)
    }

    pub fn xfunc(&self, file_name: &str, type_name: &str) -> Result<Xfunc, String> {
        let id = self.id_by_name[file_name];
        let source = self.files.source(id);
        // FIXME: don't `load` and then `.forget`
        let prg = load(source).map_err(|err| format!("{}", err))?;
        let prg = prg.forget();
        let mat = xfunc::as_matrix(&prg);
        let type_span = mat.map[type_name].info.span.unwrap();
        let impl_span = mat.map[type_name].impl_block.as_ref().and_then(|block| block.info.span);
        let title;

        let (type_text, impl_text): (String, String) = match xfunc::repr(&mat, type_name) {
            syntax::matrix::Repr::Data => {
                title = format!("Refunctionalize {}", type_name);
                let (codata, dtors, codefs) = xfunc::as_codata(&mat, type_name);

                let impl_block = ast::Impl {
                    info: ast::Info::empty(),
                    name: type_name.to_owned(),
                    defs: codefs.iter().map(|def| def.name.clone()).collect(),
                };

                let mut order = vec![codata.name.clone()];
                order.extend(codefs.iter().map(|def| def.name.clone()));

                let mut map = HashMap::default();
                map.insert(codata.name.clone(), ast::Decl::Codata(codata.clone()));
                map.extend(codefs.into_iter().map(|def| (def.name.clone(), ast::Decl::Codef(def))));
                map.extend(
                    dtors.into_iter().map(|dtor| (dtor.name.clone(), ast::Decl::Dtor(dtor))),
                );

                let decls = ast::Decls { map, order };

                let codata = codata.rename();
                let decls = decls.rename();
                let impl_block = impl_block.rename();

                let codata_string = codata.print_to_string_in_ctx(&decls);
                let impl_string = impl_block.print_to_string_in_ctx(&decls);

                (codata_string, impl_string)
            }
            syntax::matrix::Repr::Codata => {
                title = format!("Defunctionalize {}", type_name);
                let (data, ctors, defs) = xfunc::as_data(&mat, type_name);

                let impl_block = ast::Impl {
                    info: ast::Info::empty(),
                    name: type_name.to_owned(),
                    defs: defs.iter().map(|def| def.name.clone()).collect(),
                };

                let mut order = vec![data.name.clone()];
                order.extend(defs.iter().map(|def| def.name.clone()));

                let mut map = HashMap::default();
                map.insert(data.name.clone(), ast::Decl::Data(data.clone()));
                map.extend(defs.into_iter().map(|def| (def.name.clone(), ast::Decl::Def(def))));
                map.extend(
                    ctors.into_iter().map(|ctor| (ctor.name.clone(), ast::Decl::Ctor(ctor))),
                );

                let decls = ast::Decls { map, order };

                let data = data.rename();
                let decls = decls.rename();
                let impl_block = impl_block.rename();

                let data_string = data.print_to_string_in_ctx(&decls);
                let impl_string = impl_block.print_to_string_in_ctx(&decls);

                (data_string, impl_string)
            }
        };

        let edits = if let Some(impl_span) = impl_span {
            vec![
                Edit { span: type_span, text: type_text },
                Edit { span: impl_span, text: impl_text },
            ]
        } else {
            let mut type_and_impl_text = type_text;
            type_and_impl_text.push_str("\n\n");
            type_and_impl_text.push_str(&impl_text);
            vec![Edit { span: type_span, text: type_and_impl_text }]
        };

        Ok(Xfunc { title, edits })
    }

    pub fn index(&self, name: &str, location: Location) -> Option<ByteIndex> {
        let id = self.id_by_name[name];
        let line_span = self.files.line_span(id, location.line).ok()?;
        let index = line_span.start().to_usize() + location.column.to_usize();
        Some((index as u32).into())
    }

    pub fn location(&self, name: &str, idx: ByteIndex) -> Option<Location> {
        let id = self.id_by_name[name];
        self.files.location(id, idx).ok()
    }

    pub fn range(&self, name: &str, span: Span) -> Option<(Location, Location)> {
        let start = self.location(name, span.start())?;
        let end = self.location(name, span.end())?;
        Some((start, end))
    }

    pub fn info_at_index(&self, name: &str, idx: ByteIndex) -> Option<&Info> {
        self.info_at_span(name, Span::new(idx, ByteIndex(u32::from(idx) + 1)))
    }

    pub fn info_at_span(&self, name: &str, span: Span) -> Option<&Info> {
        let id = self.id_by_name[name];
        let lapper = &self.info_index_by_id[&id];
        let intervals = lapper.find(span.start().into(), span.end().into());
        let smallest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        smallest_interval.map(|interval| &self.infos_by_id[&id][interval.val])
    }

    pub fn item_at_span(&self, name: &str, span: Span) -> Option<&Item> {
        let id = self.id_by_name[name];
        let lapper = &self.item_index_by_id[&id];
        let intervals = lapper.find(span.start().into(), span.end().into());
        let largest_interval =
            intervals.max_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| &interval.val)
    }

    fn reload(&mut self, id: FileId) -> Result<(), String> {
        let source = self.files.source(id);
        match load(source) {
            Ok(ref prg) => {
                let (info_lapper, item_lapper, infos) = collect_info(prg);
                self.info_index_by_id.insert(id, info_lapper);
                self.item_index_by_id.insert(id, item_lapper);
                self.infos_by_id.insert(id, infos);
                Ok(())
            }
            Err(err) => {
                self.info_index_by_id.insert(id, Lapper::new(vec![]));
                self.item_index_by_id.insert(id, Lapper::new(vec![]));
                self.infos_by_id.insert(id, vec![]);
                Err(format!("{}", err))
            }
        }
    }
}

fn load(source: &str) -> Result<elab::Prg, Error> {
    let cst = parser::cst::parse_program(source).map_err(Error::Parser)?;
    let ast = lowering::lower(&cst).map_err(Error::Lowering)?;
    core::check(&ast).map_err(Error::Type)
}

#[derive(Default)]
struct InfoCollector {
    info_spans: Vec<Interval<u32, usize>>,
    infos: Vec<elab::TypedInfo>,
    item_spans: Vec<Interval<u32, Item>>,
}

impl Collector for InfoCollector {
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

    fn add_item_span(&mut self, item: info::Item, span: Span) {
        self.item_spans.push(Interval {
            start: span.start().into(),
            stop: span.end().into(),
            val: item,
        })
    }
}

fn collect_info(prg: &elab::Prg) -> (Lapper<u32, usize>, Lapper<u32, Item>, Vec<Info>) {
    let mut c = InfoCollector::default();

    prg.collect(&mut c);

    let info_lapper = Lapper::new(c.info_spans);
    let item_lapper = Lapper::new(c.item_spans);
    (info_lapper, item_lapper, c.infos.into_iter().map(Into::into).collect())
}

impl From<elab::TypedInfo> for Info {
    fn from(info: elab::TypedInfo) -> Self {
        Info { typ: info.typ.print_to_string(), span: info.span }
    }
}
