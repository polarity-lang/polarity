use tower_lsp::lsp_types;

pub trait ToCodespan {
    type Target;

    fn to_codespan(self) -> Self::Target;
}

pub trait ToLsp {
    type Target;

    fn to_lsp(self) -> Self::Target;
}

impl ToCodespan for lsp_types::Position {
    type Target = codespan::Location;

    fn to_codespan(self) -> Self::Target {
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
