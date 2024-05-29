use crate::PrintCfg;
use crate::PrintInCtx;

pub trait PrintToStringInCtx<C> {
    fn print_to_string_in_ctx(&self, ctx: &C) -> String;
}

impl<C, T: PrintInCtx<Ctx = C>> PrintToStringInCtx<C> for T {
    fn print_to_string_in_ctx(&self, ctx: &C) -> String {
        let alloc = super::Alloc::new();
        let mut buf = Vec::new();
        {
            let cfg = PrintCfg::default();
            let doc_builder = self.print_in_ctx(&cfg, ctx, &alloc);
            doc_builder
                .1
                .render(super::DEFAULT_WIDTH, &mut buf)
                .expect("Failed to print to string");
            drop(doc_builder)
        }
        unsafe { String::from_utf8_unchecked(buf) }
    }
}
