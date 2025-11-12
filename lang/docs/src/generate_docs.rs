use askama::Template;

use polarity_lang_ast::{Codata, Codef, Data, Decl, Def, Extern, Infix, Let, Module, Note};
use polarity_lang_printer::PrintCfg;

use crate::generate::Generate;
use crate::printer::print_html_to_string;

pub trait GenerateDocs {
    fn generate_docs(&self) -> String;
}

impl GenerateDocs for Module {
    fn generate_docs(&self) -> String {
        self.decls.iter().map(|decl| decl.generate_docs()).collect::<Vec<_>>().join("<br>")
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
            Decl::Extern(e) => e.generate_docs(),
            Decl::Infix(i) => i.generate_docs(),
            Decl::Note(n) => n.generate_docs(),
        }
    }
}

impl GenerateDocs for Data {
    fn generate_docs(&self) -> String {
        let Data { span: _, doc, name, attr, typ, ctors } = self;
        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let attr: String = print_html_to_string(attr, Some(&PrintCfg::default()));
        let typ: String = print_html_to_string(typ, Some(&PrintCfg::default()));

        let body = if ctors.is_empty() {
            "".to_string()
        } else {
            format!("<ul>{}</ul>", ctors.generate())
        };

        let data = DataTemplate { doc: &doc, name, attr: &attr, typ: &typ, body: &body };
        data.render().unwrap()
    }
}

impl GenerateDocs for Codata {
    fn generate_docs(&self) -> String {
        let Codata { span: _, doc, name, attr, typ, dtors } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let attr: String = print_html_to_string(attr, Some(&PrintCfg::default()));
        let typ: String = print_html_to_string(typ, Some(&PrintCfg::default()));

        let body = if dtors.is_empty() {
            "".to_string()
        } else {
            format!("<ul>{}</ul>", dtors.generate())
        };

        let codata = CodataTemplate { doc: &doc, name, attr: &attr, typ: &typ, body: &body };
        codata.render().unwrap()
    }
}

impl GenerateDocs for Def {
    fn generate_docs(&self) -> String {
        let Def { span: _, doc, name, attr: _, params, self_param, ret_typ, cases } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let params: String = print_html_to_string(params, Some(&PrintCfg::default()));
        let self_param: String = print_html_to_string(self_param, Some(&PrintCfg::default()));
        let ret_typ: String = print_html_to_string(ret_typ, Some(&PrintCfg::default()));

        let body = if cases.is_empty() {
            "{}".to_string()
        } else {
            format!("<ul>{}</ul>", cases.generate())
        };

        let def = DefTemplate {
            doc: &doc,
            self_param: &self_param,
            name,
            params: &params,
            typ: &ret_typ,
            body: &body,
        };
        def.render().unwrap()
    }
}

impl GenerateDocs for Codef {
    fn generate_docs(&self) -> String {
        let Codef { span: _, doc, name, attr: _, params, typ, cases } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let params: String = print_html_to_string(params, Some(&PrintCfg::default()));
        let typ: String = print_html_to_string(typ, Some(&PrintCfg::default()));

        let body = if cases.is_empty() {
            "{}".to_string()
        } else {
            format!("<ul>{}</ul>", cases.generate())
        };

        let codef = CodefTemplate { doc: &doc, name, params: &params, typ: &typ, body: &body };
        codef.render().unwrap()
    }
}

impl GenerateDocs for Let {
    fn generate_docs(&self) -> String {
        let Let { span: _, doc, name, attr: _, params, typ, body } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let params: String = print_html_to_string(params, Some(&PrintCfg::default()));
        let typ: String = print_html_to_string(typ, Some(&PrintCfg::default()));
        let body: String = print_html_to_string(body, Some(&PrintCfg::default()));

        let let_template = LetTemplate { doc: &doc, name, params: &params, typ: &typ, body: &body };
        let_template.render().unwrap()
    }
}

impl GenerateDocs for Extern {
    fn generate_docs(&self) -> String {
        let Extern { span: _, doc, name, attr: _, params, typ } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;
        let params: String = print_html_to_string(params, Some(&PrintCfg::default()));
        let typ: String = print_html_to_string(typ, Some(&PrintCfg::default()));

        let extern_template = ExternTemplate { doc: &doc, name, params: &params, typ: &typ };
        extern_template.render().unwrap()
    }
}

impl GenerateDocs for Infix {
    fn generate_docs(&self) -> String {
        let Infix { span: _, doc, attr: _, lhs, rhs } = self;
        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let lhs = print_html_to_string(lhs, Some(&PrintCfg::default()));
        let rhs = print_html_to_string(rhs, Some(&PrintCfg::default()));
        let infix_template = InfixTemplate { doc: &doc, lhs: &lhs, rhs: &rhs };
        infix_template.render().unwrap()
    }
}

impl GenerateDocs for Note {
    fn generate_docs(&self) -> String {
        let Note { span: _, doc, name, attr: _ } = self;

        let doc = if doc.is_none() { "".to_string() } else { format!("{}<br>", doc.generate()) };
        let name = &name.id;

        let note_template = NoteTemplate { doc: &doc, name };
        note_template.render().unwrap()
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
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "let.html", escape = "none")]
struct LetTemplate<'a> {
    pub doc: &'a str,
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
    pub body: &'a str,
}

#[derive(Template)]
#[template(path = "extern.html", escape = "none")]
struct ExternTemplate<'a> {
    pub doc: &'a str,
    pub name: &'a str,
    pub params: &'a str,
    pub typ: &'a str,
}

#[derive(Template)]
#[template(path = "infix.html", escape = "none")]
struct InfixTemplate<'a> {
    pub doc: &'a str,
    pub lhs: &'a str,
    pub rhs: &'a str,
}

#[derive(Template)]
#[template(path = "note.html", escape = "none")]
struct NoteTemplate<'a> {
    pub doc: &'a str,
    pub name: &'a str,
}
