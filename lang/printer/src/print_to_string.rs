use crate::PrintCfg;
use crate::PrintInCtx;

use super::Print;
use super::PrintExt;

pub trait PrintToString {
    fn print_to_string(&self, cfg: Option<&PrintCfg>) -> String;
}

impl<T: for<'a> Print<'a>> PrintToString for T {
    fn print_to_string(&self, cfg: Option<&PrintCfg>) -> String {
        let mut buf = Vec::new();
        let def = PrintCfg::default();
        let cfg = cfg.unwrap_or_else(|| &def);
        <T as PrintExt>::print(self, &cfg, &mut buf).expect("Failed to print to string");
        unsafe { String::from_utf8_unchecked(buf) }
    }
}

pub trait PrintToStringInCtx<C> {
    fn print_to_string_in_ctx(&self, ctx: &C) -> String;
}

impl<C, T: for<'a> PrintInCtx<'a, Ctx = C>> PrintToStringInCtx<C> for T {
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
