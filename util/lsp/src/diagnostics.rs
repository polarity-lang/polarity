use lsp_types::NumberOrString;
use miette::Diagnostic;
use miette::SourceSpan;
use tower_lsp::lsp_types;

use miette_util::FromMiette;
use source::DatabaseView;
use source::Error;

use crate::conversion::ToLsp;

pub trait Diagnostics {
    fn diagnostics(&self, result: Result<(), Error>) -> Vec<lsp_types::Diagnostic> {
        match result {
            Ok(()) => vec![],
            Err(err) => self.error_diagnostics(err),
        }
    }

    fn error_diagnostics(&self, error: Error) -> Vec<lsp_types::Diagnostic>;
}

impl Diagnostics for DatabaseView<'_> {
    fn error_diagnostics(&self, error: Error) -> Vec<lsp_types::Diagnostic> {
        // Compute the range where the error should be displayed.
        // The range is computed from the first available label, otherwise
        // the default range is used, which corresponds to the beginning of the
        // file.
        let span = get_span(&error);
        let range = span
            .and_then(|x| self.span_to_locations(x.from_miette()))
            .map(ToLsp::to_lsp)
            .unwrap_or_default();

        // Compute the message.
        let message = error.to_string();

        let diag = lsp_types::Diagnostic {
            range,
            message,
            severity: error.severity().map(ToLsp::to_lsp),
            code: error.code().map(|x| NumberOrString::String(format!("{x}"))),
            code_description: None,
            source: None,
            related_information: None,
            tags: None,
            data: None,
        };
        vec![diag]
    }
}

fn get_span<T: Diagnostic>(err: &T) -> Option<SourceSpan> {
    match err.labels() {
        Some(spans) => {
            let x = spans.into_iter().last();
            x.map(|y| *y.inner())
        }
        None => None,
    }
}
