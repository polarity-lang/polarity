use codespan::{ByteIndex, Location, Span};
use url::Url;

use crate::database::Database;

use super::info::{Info, Item};

impl Database {
    pub fn location_to_index(&self, uri: &Url, location: Location) -> Option<ByteIndex> {
        let file = self.files.get_even_if_stale(uri).unwrap();
        let line_span = file.line_span(location.line).ok()?;
        let index: usize = line_span.start().to_usize() + location.column.to_usize();
        Some((index as u32).into())
    }

    pub fn index_to_location(&self, uri: &Url, idx: ByteIndex) -> Option<Location> {
        let file = self.files.get_even_if_stale(uri).unwrap();
        file.location(idx).ok()
    }

    pub fn span_to_locations(&self, uri: &Url, span: Span) -> Option<(Location, Location)> {
        let start = self.index_to_location(uri, span.start())?;
        let end = self.index_to_location(uri, span.end())?;
        Some((start, end))
    }

    pub async fn hoverinfo_at_index(&mut self, uri: &Url, idx: ByteIndex) -> Option<Info> {
        self.hoverinfo_at_span(uri, Span::new(idx, ByteIndex(u32::from(idx) + 1))).await
    }

    pub async fn hoverinfo_at_span(&mut self, uri: &Url, span: Span) -> Option<Info> {
        let lapper = self.info_by_id(uri).await.ok()?;
        let intervals = lapper.find(span.start().into(), span.end().into());
        let smallest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i2.start)));
        smallest_interval.map(|interval| interval.val.clone())
    }

    pub async fn item_at_span(&mut self, uri: &Url, span: Span) -> Option<Item> {
        let lapper = self.item_by_id(uri).await.ok()?;
        let intervals = lapper.find(span.start().into(), span.end().into());
        let largest_interval =
            intervals.max_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| interval.val.clone())
    }
}
