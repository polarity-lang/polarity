mod spans;

pub trait FromLsp {
    type Target;

    #[allow(clippy::wrong_self_convention)]
    fn from_lsp(self) -> Self::Target;
}

pub trait ToLsp {
    type Target;

    fn to_lsp(self) -> Self::Target;
}
