use thiserror::Error;

use crate::{HashMap, MetaVar, MetaVarState};

pub trait Zonk {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError>;
}

#[derive(Debug, Error)]
pub enum ZonkError {
    #[error("Unbound meta-variable: ?{}", _0.id)]
    UnboundMetaVar(MetaVar),
}
