use lsp_types::NumberOrString;
use miette::Diagnostic;
use tower_lsp::lsp_types;

use data::result::Extract;
use miette_util::FromMiette;
use source::DatabaseView;
use source::Error;

use crate::conversion::ToLsp;

pub trait Diagnostics {
    fn diagnostics(&self, result: Result<(), Error>) -> Vec<lsp_types::Diagnostic> {
        result.map(|_| vec![]).map_err(|err| self.error_diagnostics(err)).extract()
    }
    fn error_diagnostics(&self, error: Error) -> Vec<lsp_types::Diagnostic>;
}

impl Diagnostics for DatabaseView<'_> {
    fn error_diagnostics(&self, error: Error) -> Vec<lsp_types::Diagnostic> {
        let labels = error.labels().unwrap_or_else(|| Box::new([].into_iter()));

        labels
            .filter_map(|label| {
                let span = label.inner().from_miette();
                let range = self.span_to_locations(span)?.to_lsp();
                let message =
                    label.label().map(ToOwned::to_owned).unwrap_or_else(|| error.to_string());
                let diag = lsp_types::Diagnostic {
                    range,
                    message,
                    severity: error.severity().map(ToLsp::to_lsp),
                    code: error.code().map(|x| NumberOrString::String(format!("{}", x))),
                    code_description: None,
                    source: None,
                    related_information: None,
                    tags: None,
                    data: None,
                };
                Some(diag)
            })
            .collect()
    }
}
