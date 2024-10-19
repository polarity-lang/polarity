pub trait ContainsMetaVars {
    /// Whether the expression or any inferred type contains metavariables
    fn contains_metavars(&self) -> bool;
}

impl<T: ContainsMetaVars> ContainsMetaVars for Vec<T> {
    fn contains_metavars(&self) -> bool {
        self.iter().any(|x| x.contains_metavars())
    }
}

impl<T: ContainsMetaVars> ContainsMetaVars for Box<T> {
    fn contains_metavars(&self) -> bool {
        self.as_ref().contains_metavars()
    }
}

impl<T: ContainsMetaVars> ContainsMetaVars for Option<T> {
    fn contains_metavars(&self) -> bool {
        self.as_ref().map_or(false, |x| x.contains_metavars())
    }
}
