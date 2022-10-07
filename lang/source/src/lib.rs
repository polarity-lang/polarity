use codespan::{ByteIndex, FileId, Files, Location, Span};
use printer::PrintToString;
use rust_lapper::{Interval, Lapper};

use data::HashMap;
use syntax::elab::{self, TypedInfo};

mod info;
mod result;

use info::{Collect, Collector};
use result::Error;

pub struct Index {
    id_by_name: HashMap<String, FileId>,
    files: Files<String>,
    info_index_by_id: HashMap<FileId, Lapper<u32, usize>>,
    infos_by_id: HashMap<FileId, Vec<Info>>,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: String,
    pub span: Option<Span>,
}

impl Index {
    pub fn empty() -> Self {
        Self {
            id_by_name: HashMap::default(),
            files: Files::new(),
            info_index_by_id: HashMap::default(),
            infos_by_id: HashMap::default(),
        }
    }

    pub fn add(&mut self, name: &str, source: String) -> FileId {
        let id = self.files.add(name, source);
        self.id_by_name.insert(name.to_owned(), id);
        self.reload(id);
        id
    }

    pub fn update(&mut self, name: &str, source: String) {
        let id = self.id_by_name[name];
        self.files.update(id, source);
        self.reload(id);
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
        let largest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| &self.infos_by_id[&id][interval.val])
    }

    fn reload(&mut self, id: FileId) {
        let source = self.files.source(id);
        let prg = load(source);
        if let Ok(ref prg) = prg {
            let (lapper, infos) = collect_info(prg);
            self.info_index_by_id.insert(id, lapper);
            self.infos_by_id.insert(id, infos);
        } else {
            self.info_index_by_id.insert(id, Lapper::new(vec![]));
            self.infos_by_id.insert(id, vec![]);
        }
    }
}

fn load(source: &str) -> Result<elab::Prg, Error> {
    let cst = parser::cst::parse_program(source).map_err(Error::Parser)?;
    let ast = lowering::lower(cst).map_err(Error::Lowering)?;
    core::check(&ast).map_err(Error::Type)
}

#[derive(Default)]
struct InfoCollector {
    intervals: Vec<Interval<u32, usize>>,
    infos: Vec<TypedInfo>,
}

impl Collector for InfoCollector {
    fn add(&mut self, _info: &elab::Info) {}

    fn add_typed(&mut self, info: &TypedInfo) {
        if let Some(span) = info.span {
            let idx = self.infos.len();
            self.intervals.push(Interval {
                start: span.start().into(),
                stop: span.end().into(),
                val: idx,
            });
            self.infos.push(info.clone());
        }
    }
}

fn collect_info(prg: &elab::Prg) -> (Lapper<u32, usize>, Vec<Info>) {
    let mut c = InfoCollector::default();

    prg.collect(&mut c);

    let lapper = Lapper::new(c.intervals);
    (lapper, c.infos.into_iter().map(Into::into).collect())
}

impl From<TypedInfo> for Info {
    fn from(info: TypedInfo) -> Self {
        Info { typ: info.typ.print_to_string(), span: info.span }
    }
}
