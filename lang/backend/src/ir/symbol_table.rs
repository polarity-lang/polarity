use ast::HashMap;

pub struct GlobalSymbolTable {
    pub modules: HashMap<String, ModuleSymbolTable>,
}

#[derive(Default)]
pub struct ModuleSymbolTable {
    pub data_decls: HashMap<String, Data>,
    pub codata_decls: HashMap<String, Codata>,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub name: String,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub name: String,
    pub dtors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub name: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub name: String,
    pub self_param: String,
    pub params: Vec<String>,
}
