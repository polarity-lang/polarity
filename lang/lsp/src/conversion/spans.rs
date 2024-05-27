use tower_lsp::lsp_types;

use super::{FromLsp, ToLsp};

impl FromLsp for lsp_types::Position {
    type Target = codespan::Location;

    fn from_lsp(self) -> Self::Target {
        codespan::Location { line: self.line.into(), column: self.character.into() }
    }
}

impl ToLsp for codespan::Location {
    type Target = lsp_types::Position;

    fn to_lsp(self) -> lsp_types::Position {
        lsp_types::Position { line: self.line.into(), character: self.column.into() }
    }
}

impl ToLsp for (codespan::Location, codespan::Location) {
    type Target = lsp_types::Range;

    fn to_lsp(self) -> Self::Target {
        lsp_types::Range { start: self.0.to_lsp(), end: self.1.to_lsp() }
    }
}
