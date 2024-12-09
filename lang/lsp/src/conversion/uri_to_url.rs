use std::str::FromStr;

use tower_lsp::lsp_types::Uri;

use super::{FromLsp, ToLsp};

impl FromLsp for &Uri {
    type Target = url::Url;

    fn from_lsp(self) -> Self::Target {
        url::Url::parse(self.as_str()).expect("Failed to parse URI")
    }
}

impl ToLsp for &url::Url {
    type Target = Uri;

    fn to_lsp(self) -> Self::Target {
        Uri::from_str(self.as_str()).expect("Failed to parse URL")
    }
}
