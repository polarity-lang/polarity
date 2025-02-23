use ast::{Case, Ctor, DocComment, Dtor};

use comrak::{markdown_to_html, Options};
use printer::{Print, PrintCfg};

static CFG: PrintCfg = PrintCfg {
    width: 100,
    latex: false,
    omit_decl_sep: false,
    de_bruijn: false,
    indent: 4,
    print_lambda_sugar: true,
    print_function_sugar: true,
    print_metavar_ids: false,
    html: true,
};
pub trait Generate {
    fn generate(&self) -> String;
}
impl Generate for Ctor {
    fn generate(&self) -> String {
        let Ctor { span: _, doc, name, params, typ } = self;
        let parameter = params.print_html_to_string(Some(&CFG));
        let typs = typ.print_html_to_string(Some(&CFG));

        let doc_str = doc.generate();
        let head = format!("{}{}", name.id, parameter);

        let head = if typ.is_simple() { head } else { format!("{}: {}", head, typs) };

        format!("<li>{}{}</li>", doc_str, head)
    }
}

impl Generate for Dtor {
    fn generate(&self) -> String {
        let Dtor { span: _, doc, name, params, self_param, ret_typ } = self;
        let self_parameter = self_param.print_html_to_string(Some(&CFG));
        let parmeter = params.print_html_to_string(Some(&CFG));
        let ret_typ = ret_typ.print_html_to_string(Some(&CFG));

        let doc_str = doc.generate();
        let head =
            if self_param.is_simple() { ".".to_owned() } else { format!("{}.", self_parameter) };

        format!("<li>{}{}{}{}: {}</li>", doc_str, head, name.id, parmeter, ret_typ)
    }
}

impl Generate for DocComment {
    fn generate(&self) -> String {
        let mut options = Options::default();
        options.render.hardbreaks = true;
        options.render.escape = true;
        let DocComment { docs } = self;
        let prefix = "<span class=\"comment\">";
        let postfix = "</span>";
        let text = docs
            .iter()
            .map(|doc| markdown_to_html(doc, &options))
            .collect::<Vec<String>>()
            .join("\n");
        format!("{}{}{}", prefix, text, postfix)
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
                format!("<li>{}</li>", value.print_html_to_string(Some(&CFG)))
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
