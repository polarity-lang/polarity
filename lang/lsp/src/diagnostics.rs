use lsp_types::NumberOrString;
use miette::Diagnostic;
use miette::SourceSpan;
use tower_lsp_server::lsp_types;
use url::Url;

use polarity_lang_driver::Database;
use polarity_lang_driver::Error;
use polarity_lang_miette_util::FromMiette;

use crate::conversion::ToLsp;

pub trait Diagnostics {
    /// Compute the diagnostics for the given URI and all of its reverse dependencies.
    async fn diagnostics(&mut self, uri: &Url, result: Result<(), Error>) -> DiagnosticsPerUri;

    fn error_diagnostics(&self, uri: &Url, error: Error) -> Vec<lsp_types::Diagnostic>;
}

pub type DiagnosticsPerUri = polarity_lang_ast::HashMap<Url, Vec<lsp_types::Diagnostic>>;

impl Diagnostics for Database {
    async fn diagnostics(&mut self, uri: &Url, result: Result<(), Error>) -> DiagnosticsPerUri {
        // When computing the diagnostics for an URI, we also need to recompute the diagnostics for all of its reverse dependencies.
        let rev_deps: Vec<_> = self.deps.reverse_dependencies(uri).into_iter().cloned().collect();
        let mut diagnostics = polarity_lang_ast::HashMap::default();

        for uri in rev_deps {
            let mut diagnostics_for_uri = vec![];
            let ast = self.ast(&uri).await;
            if let Err(err) = ast {
                diagnostics_for_uri.extend(self.error_diagnostics(&uri, err));
            }
            diagnostics.insert(uri, diagnostics_for_uri);
        }

        if let Err(err) = result {
            diagnostics.insert(uri.clone(), self.error_diagnostics(uri, err));
        } else {
            diagnostics.insert(uri.clone(), vec![]);
        }

        diagnostics
    }

    fn error_diagnostics(&self, uri: &Url, error: Error) -> Vec<lsp_types::Diagnostic> {
        if let Error::Parser(parser_errs) = error {
            return parser_errs
                .into_iter()
                .map(|e| error_diagnostics_helper(self, uri, e))
                .reduce(|mut acc, d| {
                    acc.extend(d);
                    acc
                })
                .unwrap_or_default();
        }

        error_diagnostics_helper(self, uri, error)
    }
}

fn error_diagnostics_helper<E: Diagnostic>(
    db: &Database,
    uri: &Url,
    error: E,
) -> Vec<lsp_types::Diagnostic> {
    // Compute the range where the error should be displayed.
    // The range is computed from the first available label, otherwise
    // the default range is used, which corresponds to the beginning of the
    // file.
    let span = get_span(&error);
    let range = span.and_then(|x| db.span_to_locations(uri, x.from_miette())).unwrap_or_default();

    // Compute the message.
    let message = error.to_string();

    let diag = lsp_types::Diagnostic {
        range,
        message,
        severity: match error.severity() {
            Some(sev) => Some(sev.to_lsp()),
            None => Some(lsp_types::DiagnosticSeverity::ERROR),
        },
        code: error.code().map(|x| NumberOrString::String(format!("{x}"))),
        code_description: None,
        source: None,
        related_information: None,
        tags: None,
        data: None,
    };
    vec![diag]
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
