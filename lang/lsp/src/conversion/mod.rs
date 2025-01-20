use miette::Severity;
use tower_lsp::lsp_types::DiagnosticSeverity;

mod uri_to_url;

pub trait FromLsp {
    type Target;

    #[allow(clippy::wrong_self_convention)]
    fn from_lsp(self) -> Self::Target;
}

pub trait ToLsp {
    type Target;

    fn to_lsp(self) -> Self::Target;
}

impl ToLsp for miette::Severity {
    type Target = DiagnosticSeverity;

    fn to_lsp(self) -> DiagnosticSeverity {
        match self {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Advice => DiagnosticSeverity::HINT,
        }
    }
}
