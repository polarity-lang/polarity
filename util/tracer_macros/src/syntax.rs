#[derive(Debug, Clone)]
pub struct Format {
    pub string: FormatStr,
    pub args: Vec<syn::Expr>,
    pub ret: Option<proc_macro2::TokenStream>,
    pub ret_type: Option<syn::Type>,
}

#[derive(Debug, Clone)]
pub struct FormatStr {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Separator(String),
    Whitespace(String),
    Interpolation(Arg, Option<Spec>),
}

#[derive(Debug, Clone)]
pub enum Arg {
    Return,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum Spec {
    Pretty,
    Other(String),
}

impl Format {
    pub fn expected_args_number(&self) -> usize {
        self.string.items.iter().filter(|item| matches!(item, Item::Interpolation(_, _))).count()
    }

    pub fn actual_args_number(&self) -> usize {
        self.args.len()
    }
}
