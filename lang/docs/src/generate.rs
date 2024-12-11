use ast::{Case, Ctor, DocComment, Dtor};
use printer::{Print, PrintCfg};

pub trait Generate {
    fn generate(&self) -> String;
}
impl Generate for Ctor {
    fn generate(&self) -> String {
        let Ctor { span: _, doc, name, params, typ } = self;
        let parameter = params.print_html_to_string(Some(&PrintCfg::default()));
        let typs = typ.print_html_to_string(Some(&PrintCfg::default()));

        let doc_str = doc.generate();
        let head = format!("{}{}", name.id, parameter);

        let head = if typ.is_simple() { head } else { format!("{}: {}", head, typs) };

        format!("<li>{}{}</li>", doc_str, head)
    }
}

impl Generate for Dtor {
    fn generate(&self) -> String {
        let Dtor { span: _, doc, name, params, self_param, ret_typ } = self;
        let self_parameter = self_param.print_html_to_string(Some(&PrintCfg::default()));
        let parmeter = params.print_html_to_string(Some(&PrintCfg::default()));
        let ret_typ = ret_typ.print_html_to_string(Some(&PrintCfg::default()));

        let doc_str = doc.generate();
        let head =
            if self_param.is_simple() { ".".to_owned() } else { format!("{}.", self_parameter) };

        format!("<li>{}{}{}{}: {}</li>", doc_str, head, name.id, parmeter, ret_typ)
    }
}

impl Generate for DocComment {
    fn generate(&self) -> String {
        let DocComment { docs } = self;
        let prefix = "<span class=\"comment\">";
        let postfix = "</span>";
        docs.iter()
            .map(|doc| {
                format!(
                    "{} {} {}",
                    prefix,
                    askama_escape::escape(doc, askama_escape::Html),
                    postfix
                )
            })
            .collect::<Vec<String>>()
            .join("<br>")
            + "<br>"
    }
}

impl<T: Generate> Generate for Option<T> {
    fn generate(&self) -> String {
        match self {
            Some(value) => value.generate(),
            None => "".to_string(),
        }
    }
}

impl Generate for Vec<DocComment> {
    fn generate(&self) -> String {
        self.iter().map(|value| value.generate()).collect::<Vec<String>>().join(",<br>")
    }
}

impl Generate for Vec<Case> {
    fn generate(&self) -> String {
        self.iter()
            .map(|value| {
                format!("<li>{}</li>", value.print_html_to_string(Some(&PrintCfg::default())))
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

impl Generate for Vec<Ctor> {
    fn generate(&self) -> String {
        self.iter().map(|value| value.generate()).collect::<Vec<String>>().join("")
    }
}

impl Generate for Vec<Dtor> {
    fn generate(&self) -> String {
        self.iter().map(|value| value.generate()).collect::<Vec<String>>().join("")
    }
}

impl<T: Generate> Generate for Box<T> {
    fn generate(&self) -> String {
        self.as_ref().generate()
    }
}
