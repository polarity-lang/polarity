use ast::{Codata, Codef, Data, Decl, Def, Let, Module,};
use askama::Template;

use crate::generate::Generate;
pub trait GenerateDocs {
    fn generate_docs(&self) -> String;
}

impl GenerateDocs for Module {
    fn generate_docs(&self) -> String {
        self.decls.iter()
            .map(|decl| decl.generate_docs())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl GenerateDocs for Decl {
    fn generate_docs(&self) -> String {
        match self {
            Decl::Data(data) => data.generate_docs(),
            Decl::Codata(codata) => codata.generate_docs(),
            Decl::Def(def) => def.generate_docs(),
            Decl::Codef(codef) => codef.generate_docs(),
            Decl::Let(l) => l.generate_docs(),
        }
    }
}

impl GenerateDocs for Data {
    fn generate_docs(&self) -> String {
        let Data { span: _, doc, name, attr, typ, ctors } = self;
        let doc = doc.generate();
        let name = &name.id;
        let attr: String = attr.generate();
        let typ: String = typ.generate();

        let body = if ctors.is_empty() {
            "{}".to_string()
        } else {
            ctors.generate()
        };

        let data = DataTemplate {
            doc: &doc,
            name: &name,
            attr: &attr,
            typ: &typ,
            body: &body,
        };
        data.render().unwrap()
    }
}

impl GenerateDocs for Codata {
    fn generate_docs(&self) -> String {
        let Codata { span: _, doc, name, attr, typ, dtors } = self;

        let doc = doc.generate();
        let name = &name.id;
        let attr: String = attr.generate();
        let typ: String = typ.generate();

        let body = if dtors.is_empty() {
            "{}".to_string()
        } else {
            dtors.generate()
        };

        let codata = CodataTemplate {
            doc: &doc,
            name: &name,
            attr: &attr,
            typ: &typ,
            body: &body,
        };
        codata.render().unwrap()
    }
}

impl GenerateDocs for Def {
    fn generate_docs(&self) -> String {
        let Def { span: _, doc, name, attr, params, self_param, ret_typ, cases } = self;

        let doc = doc.generate();
        let name = &name.id;
        let params: String = params.generate();
        let self_param: String = self_param.generate();
        let ret_typ: String = ret_typ.generate();
        let cases: String = cases.generate();

        let body = if cases.is_empty() {
            "{}".to_string()
        } else {
            cases
        };

        let def = DefTemplate {
            doc: &doc,
            self_param: &self_param,
            name: &name,
            params: &params,
            typ: &ret_typ,
            body: &body,
        };
        def.render().unwrap()
    }
}

impl GenerateDocs for Codef {
    fn generate_docs(&self) -> String {
        let Codef { span: _, doc, name, attr, params, typ, cases } = self;

        let doc = doc.generate();
        let name = &name.id;
        let params: String = params.generate();
        let typ: String = typ.generate();
        let cases: String = cases.generate();

        let body = if cases.is_empty() {
            "{}".to_string()
        } else {
            cases
        };

        let codef = CodefTemplate {
            doc: &doc,
            self_param: "",
            name: &name,
            params: &params,
            typ: &typ,
            body: &body,
        };
        codef.render().unwrap()
    }
}

impl GenerateDocs for Let {
    fn generate_docs(&self) -> String {
        let Let { span: _, doc, name, attr, params, typ, body } = self;

        let doc = doc.generate();
        let name = &name.id;
        let attr: String = attr.generate();
        let params: String = params.generate();
        let typ: String = typ.generate();
        let body: String = body.generate();

        let let_template = LetTemplate {
            doc: &doc,
            self_param: "",
            name: &name,
            params: &params,
            typ: &typ,
            body: &body,
        };
        let_template.render().unwrap()
    }
}

#[derive(Template)]
#[template(path = "data.html", escape = "none")]
struct DataTemplate<'a> {
    pub doc: &'a str,
    pub name: &'a str,
    pub attr: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "codata.html", escape = "none")]
struct CodataTemplate<'a> {
    pub doc: &'a str,
    pub name: &'a str,
    pub attr: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "def.html", escape = "none")]
struct DefTemplate<'a> {
    pub doc: &'a str,
    pub self_param: &'a str,
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "codef.html", escape = "none")]
struct CodefTemplate<'a> {
    pub doc: &'a str,
    pub self_param: &'a str,
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "let.html", escape = "none")]
struct LetTemplate<'a> {
    pub doc: &'a str,
    pub self_param: &'a str,
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}





