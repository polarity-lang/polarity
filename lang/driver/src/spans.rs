use lsp_types::{Position, Range};
use miette_util::codespan::{ByteIndex, LineIndex, Span};
use url::Url;

use crate::database::Database;

use super::info::{Info, Item};

impl Database {
    pub fn location_to_index(&self, uri: &Url, location: Position) -> Option<ByteIndex> {
        let file = self.files.get_even_if_stale(uri).unwrap();
        let line_span = file.line_span(LineIndex(location.line)).ok()?;
        let index: usize = line_span.start().to_usize() + LineIndex(location.character).to_usize();
        Some(ByteIndex(index as u32))
    }

    pub fn index_to_location(&self, uri: &Url, idx: ByteIndex) -> Option<Position> {
        let file = self.files.get_even_if_stale(uri).unwrap();
        file.location(idx).ok()
    }

    pub fn span_to_locations(&self, uri: &Url, span: Span) -> Option<Range> {
        let start = self.index_to_location(uri, span.start())?;
        let end = self.index_to_location(uri, span.end())?;
        Some(Range { start, end })
    }

    pub async fn hoverinfo_at_index(&mut self, uri: &Url, idx: ByteIndex) -> Option<Info> {
        self.hoverinfo_at_span(uri, Span::new(idx, ByteIndex(idx.0 + 1))).await
    }

    pub async fn hoverinfo_at_span(&mut self, uri: &Url, span: Span) -> Option<Info> {
        let lapper = self.info_by_id(uri).await.ok()?;
        let intervals = lapper.find(span.start().0, span.end().0);
        let smallest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i2.start)));
        smallest_interval.map(|interval| interval.val.clone())
    }

    pub async fn item_at_span(&mut self, uri: &Url, span: Span) -> Option<Item> {
        let lapper = self.item_by_id(uri).await.ok()?;
        let intervals = lapper.find(span.start().0, span.end().0);
        let largest_interval =
            intervals.max_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| interval.val.clone())
    }
}
