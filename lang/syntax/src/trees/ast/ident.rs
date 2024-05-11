use derivative::Derivative;

pub type Ident = String;

/// A metavariable which stands for unknown terms which
/// have to be determined during elaboration.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct MetaVar {
    pub id: u64,
}
