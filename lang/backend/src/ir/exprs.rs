#[derive(Debug, Clone)]
pub enum Exp {
    Variable(Variable),
    CtorCall(Call),
    CodefCall(Call),
    LetCall(Call),
    DtorCall(DotCall),
    DefCall(DotCall),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Panic(Panic),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub name: String,
    pub args: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct DotCall {
    pub exp: Box<Exp>,
    pub name: String,
    pub args: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct LocalMatch {
    pub on_exp: Box<Exp>,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct LocalComatch {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Panic {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub pattern: Pattern,
    pub body: Option<Box<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub is_copattern: bool,
    pub name: String,
    pub params: Vec<String>,
}
