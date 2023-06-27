use crate::cst::Info;

pub fn span(l: usize, r: usize) -> Info {
    Info::spanned(l as u32, r as u32)
}
