use crate::RenameInfo;

use codespan::Span;

impl RenameInfo for Option<Span> {
    fn rename_in_ctx(self, _ctx: &mut crate::Ctx) -> Self {
        self
    }
}

impl RenameInfo for () {
    fn rename_in_ctx(self, _ctx: &mut crate::Ctx) -> Self {
        self
    }
}
