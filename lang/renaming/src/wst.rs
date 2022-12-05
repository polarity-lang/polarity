use crate::{Rename, RenameInfo};

use syntax::wst;

impl RenameInfo for wst::TypeInfo {
    fn rename_in_ctx(self, ctx: &mut crate::Ctx) -> Self {
        let wst::TypeInfo { typ, span } = self;

        let typ = typ.rename_in_ctx(ctx);

        wst::TypeInfo { typ, span }
    }
}

impl RenameInfo for wst::TypeAppInfo {
    fn rename_in_ctx(self, ctx: &mut crate::Ctx) -> Self {
        let wst::TypeAppInfo { typ, span } = self;

        let typ = typ.rename_in_ctx(ctx);

        wst::TypeAppInfo { typ, span }
    }
}
