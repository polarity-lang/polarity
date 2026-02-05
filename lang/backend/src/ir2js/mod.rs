//! JavaScript code generation using SWC.

use std::io;
use std::rc::Rc;

use swc_common::SourceMap;
use swc_ecma_ast as js;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config as CodegenConfig, Emitter};

mod decls;
mod exprs;
mod tokens;
mod traits;

use crate::result::{BackendError, BackendResult};

use super::ir;

/// Convert an IR module to JavaScript
pub fn ir_to_js<W: io::Write>(ir_module: &ir::Module, writer: W) -> BackendResult {
    let call_to_main = ir_module.has_main();
    let js_module = ir_module.to_js_module()?;
    emit_js(&js_module, writer, call_to_main)
}

/// Emit a JavaScript module
fn emit_js<W: io::Write>(
    js_module: &js::Module,
    mut writer: W,
    call_to_main: bool,
) -> BackendResult {
    let config = CodegenConfig::default();
    let cm = Rc::new(SourceMap::default());

    let js_writer = JsWriter::new(cm.clone(), "\n", &mut writer, None);
    let mut emitter = Emitter { cfg: config, cm, comments: None, wr: Box::new(js_writer) };

    emitter
        .emit_module(js_module)
        .map_err(|e| BackendError::CodegenError(format!("Failed to emit module: {}", e)))?;

    // NOTE: This is a temporary solution until we have IO support
    if call_to_main {
        write!(writer, "\nconsole.log(JSON.stringify(main()))").map_err(|e| {
            BackendError::CodegenError(format!("Failed to write call to main: {e}"))
        })?;
    }

    Ok(())
}
