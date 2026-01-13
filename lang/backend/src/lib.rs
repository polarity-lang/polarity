pub mod ast2ir;
pub mod ir;
pub mod ir2js;
pub mod result;

pub use ir::rename::{rename_ir, rename_ir_for_js};
pub use ir2js::ir_to_js;

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Javascript,
}
