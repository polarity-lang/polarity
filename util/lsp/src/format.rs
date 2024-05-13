//! Implementation of the formatting functionality of the LSP server
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::server::*;
use printer::{PrintCfg, PrintToString};

pub async fn formatting(
    server: &Server,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let text_document = params.text_document;

    server
        .client
        .log_message(MessageType::INFO, format!("Formatting request: {}", text_document.uri))
        .await;

    let db = server.database.read().await;
    let index = db.get(text_document.uri.as_str()).unwrap();
    let prg = match index.ust() {
        Ok(prg) => prg,
        Err(_) => return Ok(None),
    };

    let rng: Range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: u32::MAX, character: u32::MAX },
    };

    let cfg = PrintCfg {
        width: 100,
        latex: false,
        omit_decl_sep: false,
        de_bruijn: false,
        indent: 4,
        print_lambda_sugar: true,
        print_function_sugar: true,
    };

    let formatted_prog: String = prg.print_to_string(Some(&cfg));

    let text_edit: TextEdit = TextEdit { range: rng, new_text: formatted_prog };

    Ok(Some(vec![text_edit]))
}
