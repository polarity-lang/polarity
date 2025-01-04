use miette_util::codespan::{ColumnIndex, LineIndex};
use tower_lsp::lsp_types;

use super::{FromLsp, ToLsp};

impl FromLsp for lsp_types::Position {
    type Target = miette_util::codespan::Location;

    fn from_lsp(self) -> Self::Target {
        miette_util::codespan::Location {
            line: LineIndex(self.line),
            column: ColumnIndex(self.character),
        }
    }
}

impl ToLsp for miette_util::codespan::Location {
    type Target = lsp_types::Position;

    fn to_lsp(self) -> lsp_types::Position {
        lsp_types::Position { line: self.line.0, character: self.column.0 }
    }
}

impl ToLsp for (miette_util::codespan::Location, miette_util::codespan::Location) {
    type Target = lsp_types::Range;

    fn to_lsp(self) -> Self::Target {
        lsp_types::Range { start: self.0.to_lsp(), end: self.1.to_lsp() }
    }
}
