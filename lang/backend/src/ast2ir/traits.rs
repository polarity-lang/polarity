use crate::ir;
use crate::result::BackendResult;

/// Convert AST to IR (intermediate representation)
///
/// Takes into account the erasure information annotated in the AST.
/// Nodes annotated with `erased: true` won't occur in the generated IR.
pub trait ToIR {
    type Target;

    fn to_ir(&self) -> BackendResult<Self::Target>;
}

impl<T: ToIR> ToIR for Vec<T> {
    type Target = Vec<T::Target>;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        self.iter().map(|x| x.to_ir()).collect()
    }
}

impl<T: ToIR> ToIR for Option<T> {
    type Target = Option<T::Target>;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        match self {
            Some(x) => Ok(Some(x.to_ir()?)),
            None => Ok(None),
        }
    }
}

pub trait CollectToplevelNames {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>);
}

impl<T: CollectToplevelNames> CollectToplevelNames for Vec<T> {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        self.iter().for_each(|x| x.collect_toplevel_names(names));
    }
}
