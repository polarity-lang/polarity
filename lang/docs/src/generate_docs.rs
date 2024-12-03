use ast::{Codata, Codef, Data, Decl, Def, Let, Module};

pub trait GenerateDocs {
    fn generate_docs(&self) -> String;
}

impl GenerateDocs for Module{
    fn generate_docs(&self) -> String {
        "module".to_string()
        //todo generate html for all generations and concat them
        }
}

impl GenerateDocs for Decl{
    fn generate_docs(&self) -> String {
        match self{
            Decl::Data(data) => data.generate_docs(),
            Decl::Codata(codata) => codata.generate_docs(),
            Decl::Def(def) => def.generate_docs(),
            Decl::Codef(codef) => codef.generate_docs(),
            Decl::Let(l) => l.generate_docs(),
        }
    }
}

impl GenerateDocs for Data{
    fn generate_docs(&self) -> String {
        "data".to_string()
    }
}

impl GenerateDocs for Codata{
    fn generate_docs(&self) -> String {
        "codata".to_string()
    }
}

impl GenerateDocs for Def{
    fn generate_docs(&self) -> String {
        "def".to_string()
    }
}

impl GenerateDocs for Codef{
    fn generate_docs(&self) -> String {
        "codef".to_string()
    }
}

impl GenerateDocs for Let{
    fn generate_docs(&self) -> String {
        "let".to_string()
    }
}