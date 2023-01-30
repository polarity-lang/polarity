use crate::RenameInfo;

use syntax::ust;

impl RenameInfo for ust::Info {
    fn rename_in_ctx(self, _ctx: &mut crate::Ctx) -> Self {
        self
    }
}
