use std::io;

use printer::{Alloc, Print, PrintCfg};

use crate::render;

pub fn print_html<W: io::Write, P: Print>(pr: P, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
    let alloc = Alloc::new();
    let doc_builder = pr.print(cfg, &alloc);
    doc_builder.render_raw(cfg.width, &mut render::RenderHtml::new(out))
}

pub fn print_html_to_string<P: Print>(pr: P, cfg: Option<&PrintCfg>) -> String {
    let mut buf = Vec::new();
    let def = PrintCfg::default();
    let cfg = cfg.unwrap_or(&def);
    print_html(pr, cfg, &mut buf).expect("Failed to print to string");
    String::from_utf8(buf).expect("Failed to convert Vec<u8> to String")
}
