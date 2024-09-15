use codespan::{ByteIndex, Location, Span};
use rust_lapper::Lapper;

use super::info::{Info, Item};
use super::DatabaseViewMut;

impl<'a> DatabaseViewMut<'a> {
    pub fn location_to_index(&self, location: Location) -> Option<ByteIndex> {
        let DatabaseViewMut { uri: url, database } = self;
        let file = database.files.get_even_if_stale(url).unwrap();
        let line_span = file.line_span(location.line).ok()?;
        let index: usize = line_span.start().to_usize() + location.column.to_usize();
        Some((index as u32).into())
    }

    pub fn index_to_location(&self, idx: ByteIndex) -> Option<Location> {
        let DatabaseViewMut { uri: url, database } = self;
        let file = database.files.get_even_if_stale(url).unwrap();
        file.location(idx).ok()
    }

    pub fn span_to_locations(&self, span: Span) -> Option<(Location, Location)> {
        let start = self.index_to_location(span.start())?;
        let end = self.index_to_location(span.end())?;
        Some((start, end))
    }

    pub fn hoverinfo_at_index(&self, idx: ByteIndex) -> Option<Info> {
        self.hoverinfo_at_span(Span::new(idx, ByteIndex(u32::from(idx) + 1)))
    }

    pub fn hoverinfo_at_span(&self, span: Span) -> Option<Info> {
        let lapper = self.infos();
        let intervals = lapper.find(span.start().into(), span.end().into());
        let smallest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i2.start)));
        smallest_interval.map(|interval| interval.val.clone())
    }

    pub fn item_at_span(&self, span: Span) -> Option<Item> {
        let lapper = self.items();
        let intervals = lapper.find(span.start().into(), span.end().into());
        let largest_interval =
            intervals.max_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| interval.val.clone())
    }

    pub fn infos(&self) -> &Lapper<u32, Info> {
        &self.database.info_by_id[&self.uri]
    }

    pub fn items(&self) -> &Lapper<u32, Item> {
        &self.database.item_by_id[&self.uri]
    }
}
