use codespan::{ByteIndex, Location, Span};

use super::info::{HoverInfo, Item};
use super::DatabaseView;

impl<'a> DatabaseView<'a> {
    pub fn location_to_index(&self, location: Location) -> Option<ByteIndex> {
        let DatabaseView { file_id, database } = self;
        let line_span = database.files.line_span(*file_id, location.line).ok()?;
        let index = line_span.start().to_usize() + location.column.to_usize();
        Some((index as u32).into())
    }

    pub fn index_to_location(&self, idx: ByteIndex) -> Option<Location> {
        let DatabaseView { file_id, database } = self;
        database.files.location(*file_id, idx).ok()
    }

    pub fn span_to_locations(&self, span: Span) -> Option<(Location, Location)> {
        let start = self.index_to_location(span.start())?;
        let end = self.index_to_location(span.end())?;
        Some((start, end))
    }

    pub fn hoverinfo_at_index(&self, idx: ByteIndex) -> Option<HoverInfo> {
        self.hoverinfo_at_span(Span::new(idx, ByteIndex(u32::from(idx) + 1)))
    }

    pub fn hoverinfo_at_span(&self, span: Span) -> Option<HoverInfo> {
        let index = self.index()?;
        let lapper = index.infos();
        let intervals = lapper.find(span.start().into(), span.end().into());
        let smallest_interval =
            intervals.min_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        smallest_interval.map(|interval| interval.val.clone())
    }

    pub fn item_at_span(&self, span: Span) -> Option<Item> {
        let index = self.index()?;
        let lapper = index.items();
        let intervals = lapper.find(span.start().into(), span.end().into());
        let largest_interval =
            intervals.max_by(|i1, i2| (i1.stop - i1.start).cmp(&(i2.stop - i1.start)));
        largest_interval.map(|interval| interval.val.clone())
    }
}
