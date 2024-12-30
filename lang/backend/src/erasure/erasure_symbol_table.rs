//! Symbol table to look up top-level definitions during erasure

use url::Url;

use ast::HashMap;

use super::result::ErasureError;

pub fn build_erasure_symbol_table(module: &ast::Module) -> ModuleErasureSymbolTable {
    let mut table = ModuleErasureSymbolTable::default();
    for decl in module.decls.iter() {
        match decl {
            ast::Decl::Data(data) => data.build(&mut table),
            ast::Decl::Codata(codata) => codata.build(&mut table),
            ast::Decl::Def(def) => def.build(&mut table),
            ast::Decl::Codef(codef) => codef.build(&mut table),
            ast::Decl::Let(tl_let) => tl_let.build(&mut table),
        }
    }
    table
}

pub struct GlobalErasureSymbolTable {
    pub modules: HashMap<String, ModuleErasureSymbolTable>,
}

impl GlobalErasureSymbolTable {
    pub fn lookup_data(&self, uri: &Url, name: &str) -> Result<&Data, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_data
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown data type: {}", name)))
    }

    pub fn lookup_codata(&self, uri: &Url, name: &str) -> Result<&Codata, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_codata
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown codata type: {}", name)))
    }

    pub fn lookup_def(&self, uri: &Url, name: &str) -> Result<&DefMeta, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_def
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown definition: {}", name)))
    }

    pub fn lookup_codef(&self, uri: &Url, name: &str) -> Result<&CodefMeta, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_codef
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown codefinition: {}", name)))
    }

    pub fn lookup_let(&self, uri: &Url, name: &str) -> Result<&LetMeta, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_let
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown let binding: {}", name)))
    }

    pub fn lookup_ctor(&self, uri: &Url, name: &str) -> Result<&Ctor, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_ctor
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown constructor: {}", name)))
    }

    pub fn lookup_dtor(&self, uri: &Url, name: &str) -> Result<&Dtor, ErasureError> {
        let module = self.lookup_module(uri)?;
        module
            .map_dtor
            .get(name)
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown destructor: {}", name)))
    }

    fn lookup_module(&self, uri: &Url) -> Result<&ModuleErasureSymbolTable, ErasureError> {
        self.modules
            .get(uri.as_str())
            .ok_or_else(|| ErasureError::Impossible(format!("Unknown module: {}", uri.as_str())))
    }
}

#[derive(Default)]
pub struct ModuleErasureSymbolTable {
    pub map_data: HashMap<String, Data>,
    pub map_codata: HashMap<String, Codata>,
    pub map_def: HashMap<String, DefMeta>,
    pub map_codef: HashMap<String, CodefMeta>,
    pub map_let: HashMap<String, LetMeta>,
    pub map_ctor: HashMap<String, Ctor>,
    pub map_dtor: HashMap<String, Dtor>,
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
    pub params: Vec<Param>,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub name: String,
    pub self_param: String,
    pub params: Vec<Param>,
    pub erased: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub erased: bool,
}

#[derive(Debug, Clone)]
pub struct DefMeta {
    pub name: String,
    pub self_param: String,
    pub params: Vec<Param>,
    pub erased: bool,
}

#[derive(Debug, Clone)]
pub struct CodefMeta {
    pub name: String,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone)]
pub struct LetMeta {
    pub name: String,
    pub params: Vec<Param>,
    pub erased: bool,
}

pub trait BuildSymbolTable {
    fn build(&self, table: &mut ModuleErasureSymbolTable);
}

impl BuildSymbolTable for ast::Data {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Data { span: _, doc: _, name, attr: _, typ: _, ctors } = self;

        let ctors = ctors
            .iter()
            .map(|ast::Ctor { span: _, doc: _, name, params, typ: _ }| {
                let params = mark_erased_params(&params.params);
                Ctor { name: name.to_string(), params }
            })
            .collect();

        let data = Data { name: name.to_string(), ctors };
        table.map_data.insert(name.to_string(), data);
    }
}

impl BuildSymbolTable for ast::Codata {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Codata { span: _, doc: _, name, attr: _, typ: _, dtors } = self;

        let dtors = dtors
            .iter()
            .map(|ast::Dtor { span: _, doc: _, name, params, self_param, ret_typ }| {
                let params = mark_erased_params(&params.params);
                let erased = is_erased_type(ret_typ);
                Dtor {
                    name: name.to_string(),
                    self_param: self_param
                        .name
                        .as_ref()
                        .map(|nm| nm.to_string())
                        .unwrap_or_default(),
                    params,
                    erased,
                }
            })
            .collect();

        let codata = Codata { name: name.to_string(), dtors };
        table.map_codata.insert(name.to_string(), codata);
    }
}

impl BuildSymbolTable for ast::Ctor {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Ctor { span: _, doc: _, name, params, typ: _ } = self;

        let params = mark_erased_params(&params.params);
        let ctor = Ctor { name: name.to_string(), params };
        table.map_ctor.insert(name.to_string(), ctor);
    }
}

impl BuildSymbolTable for ast::Dtor {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Dtor { span: _, doc: _, name, params, self_param, ret_typ } = self;

        let params = mark_erased_params(&params.params);
        let erased = is_erased_type(ret_typ);
        let dtor = Dtor {
            name: name.to_string(),
            self_param: self_param.name.as_ref().map(|nm| nm.to_string()).unwrap_or_default(),
            params,
            erased,
        };
        table.map_dtor.insert(name.to_string(), dtor);
    }
}

impl BuildSymbolTable for ast::Def {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Def { span: _, doc: _, name, attr: _, params, self_param, ret_typ, cases: _ } =
            self;

        let params = mark_erased_params(&params.params);
        let erased = is_erased_type(ret_typ);

        let def = DefMeta {
            name: name.to_string(),
            self_param: self_param.name.as_ref().map(|nm| nm.to_string()).unwrap_or_default(),
            params,
            erased,
        };
        table.map_def.insert(name.to_string(), def);
    }
}

impl BuildSymbolTable for ast::Codef {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Codef { span: _, doc: _, name, attr: _, params, typ: _, cases: _ } = self;

        let params = mark_erased_params(&params.params);

        let codef = CodefMeta { name: name.to_string(), params };
        table.map_codef.insert(name.to_string(), codef);
    }
}

impl BuildSymbolTable for ast::Let {
    fn build(&self, table: &mut ModuleErasureSymbolTable) {
        let ast::Let { span: _, doc: _, name, attr: _, params, typ, body: _ } = self;

        let params = mark_erased_params(&params.params);
        let erased = is_erased_type(typ);

        let let_meta = LetMeta { name: name.to_string(), params, erased };
        table.map_let.insert(name.to_string(), let_meta);
    }
}

/// Mark parameters as erased where applicable
fn mark_erased_params(params: &[ast::Param]) -> Vec<Param> {
    params.iter().map(mark_erased_param).collect()
}

/// Mark a parameter as erased if applicable
fn mark_erased_param(param: &ast::Param) -> Param {
    let erased = is_erased_type(&param.typ);

    Param { name: param.name.to_string(), erased }
}

/// Whether a term of type `typ: Type` can be erased.
fn is_erased_type(typ: &ast::Exp) -> bool {
    match typ {
        ast::Exp::Variable(_) => false,
        ast::Exp::TypCtor(_) => false,
        ast::Exp::Call(_) => false,
        ast::Exp::DotCall(_) => false,
        ast::Exp::Anno(anno) => is_erased_type(&anno.exp),
        ast::Exp::TypeUniv(_) => true,
        ast::Exp::LocalMatch(_) => false,
        ast::Exp::LocalComatch(_) => false,
        ast::Exp::Hole(_) => false,
    }
}
