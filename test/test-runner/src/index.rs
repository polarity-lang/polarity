use tantivy::collector::DocSetCollector;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{LeasedItem, ReloadPolicy};

use super::suites::Case;

// 50 MB
const INDEX_WRITER_MEM: usize = 50_000_000;

pub struct Index {
    index: tantivy::Index,
    reader: tantivy::IndexReader,
}

pub struct Writer<'a> {
    index: &'a mut Index,
    writer: tantivy::IndexWriter,
}

pub struct Searcher {
    searcher: LeasedItem<tantivy::Searcher>,
}

impl Index {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("suite", TEXT | STORED);
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("path", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);

        let schema = schema_builder.build();

        let index = tantivy::Index::create_in_ram(schema);

        let reader = index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into().unwrap();

        Self { index, reader }
    }

    pub fn writer(&'_ mut self) -> Writer<'_> {
        let index_writer = self.index.writer(INDEX_WRITER_MEM).unwrap();
        Writer { index: self, writer: index_writer }
    }

    pub fn searcher(&self) -> Searcher {
        let searcher = self.reader.searcher();
        Searcher { searcher }
    }

    fn reload(&mut self) {
        self.reader =
            self.index.reader_builder().reload_policy(ReloadPolicy::Manual).try_into().unwrap();
    }
}

impl Writer<'_> {
    pub fn add(&mut self, suite: &str, case: &Case, content: &str) {
        let schema = self.writer.index().schema();
        let suite_field = schema.get_field("suite").unwrap();
        let name_field = schema.get_field("name").unwrap();
        let path_field = schema.get_field("path").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let mut doc = Document::default();
        doc.add_text(suite_field, suite);
        doc.add_text(name_field, &case.name);
        doc.add_text(path_field, case.path.to_str().unwrap());
        doc.add_text(content_field, content);
        self.writer.add_document(doc).unwrap();
    }

    pub fn commit(&mut self) {
        self.writer.commit().unwrap();
        self.index.reload();
    }
}

impl Searcher {
    pub fn search(&self, q: &str) -> impl Iterator<Item = Case> {
        let schema = self.searcher.index().schema();

        let suite_field = schema.get_field("suite").unwrap();
        let name_field = schema.get_field("name").unwrap();
        let path_field = schema.get_field("path").unwrap();
        let content_field = schema.get_field("content").unwrap();

        let fields = vec![suite_field, name_field, path_field, content_field];

        let query_parser = QueryParser::for_index(self.searcher.index(), fields);
        let query = query_parser.parse_query(q).unwrap();

        let res = self.searcher.search(&query, &DocSetCollector).unwrap().into_iter().map(|addr| {
            let doc = self.searcher.doc(addr).unwrap();
            Case {
                suite: doc.get_first(suite_field).unwrap().as_text().unwrap().to_owned(),
                name: doc.get_first(name_field).unwrap().as_text().unwrap().to_owned(),
                path: doc.get_first(path_field).unwrap().as_text().unwrap().into(),
            }
        });

        res.collect::<Vec<_>>().into_iter()
    }
}
