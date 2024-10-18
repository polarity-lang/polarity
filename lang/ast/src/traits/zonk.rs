use thiserror::Error;

use crate::{HashMap, MetaVar, MetaVarState};

/// Insert metavariable solutions in all holes in the AST
pub trait Zonk {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError>;
}

impl<T: Zonk> Zonk for Option<T> {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError> {
        if let Some(inner) = self {
            inner.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl<T: Zonk> Zonk for Box<T> {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError> {
        self.as_mut().zonk(meta_vars)
    }
}

impl<T: Zonk> Zonk for Vec<T> {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError> {
        for item in self {
            item.zonk(meta_vars)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ZonkError {
    #[error("Unbound meta-variable: ?{}", _0.id)]
    UnboundMetaVar(MetaVar),
}
