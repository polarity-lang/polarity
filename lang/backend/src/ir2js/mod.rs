//! JavaScript code generation using SWC.

use std::io;
use std::rc::Rc;

use swc_core::common::SourceMap;
use swc_core::ecma::ast as js;
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::codegen::{Config as CodegenConfig, Emitter};
use swc_core::quote;

mod decls;
mod exprs;
mod tokens;
mod traits;
mod util;

use crate::result::{BackendError, BackendResult};

use super::ir;

enum CallToMain {
    None,
    RunIO,
    DebugPrint,
}

impl CallToMain {
    fn generated_call(&self) -> Option<js::Stmt> {
        match self {
            CallToMain::None => None,
            CallToMain::RunIO => Some(quote!("main()()" as Stmt)),
            CallToMain::DebugPrint => Some(quote!(
                "console.log(JSON.stringify(main(), (k, v) => typeof v == \"bigint\" ? v.toString() : v))"
                    as Stmt
            )),
        }
    }
}

/// Convert an IR module to JavaScript
pub fn ir_to_js<W: io::Write>(ir_module: &ir::Module, writer: W) -> BackendResult {
    let mut js_module = ir_module.to_js_module()?;

    let call_to_main = ir_module.find_main().map_or(CallToMain::None, |main| {
        if main.is_main_with_io { CallToMain::RunIO } else { CallToMain::DebugPrint }
    });
    if let Some(call) = call_to_main.generated_call() {
        js_module.body.push(js::ModuleItem::Stmt(call));
    }

    emit_js(&js_module, writer)
}

/// Emit a JavaScript module
fn emit_js<W: io::Write>(js_module: &js::Module, mut writer: W) -> BackendResult {
    let config = CodegenConfig::default();
    let cm = Rc::new(SourceMap::default());

    let js_writer = JsWriter::new(cm.clone(), "\n", &mut writer, None);
    let mut emitter = Emitter { cfg: config, cm, comments: None, wr: Box::new(js_writer) };

    emitter
        .emit_module(js_module)
        .map_err(|e| BackendError::CodegenError(format!("Failed to emit module: {}", e)))?;

    Ok(())
}
