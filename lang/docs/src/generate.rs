use ast::{
    shift_and_clone, Arg, Args, Case, Ctor, DocComment, Dtor, Exp, Param, Telescope, TypCtor,
};

use comrak::{markdown_to_html, Options};
use printer::{Print, PrintCfg};

pub trait Generate {
    fn generate(&self) -> String;
}
impl Generate for Ctor {
    fn generate(&self) -> String {
        let Ctor { span: _, doc, name, params, typ } = self;
        let parameter = params.generate();
        let typs = typ.generate();

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
        let parmeter = params.generate();
        let ret_typ = ret_typ.generate();

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

impl Generate for Telescope {
    fn generate(&self) -> String {
        let Telescope { params } = self;
        if params.is_empty() {
            return String::new();
        }

        let mut running: Option<(&Exp, bool)> = None;
        let mut output = String::new();

        for Param { implicit, name, typ, erased: _ } in params {
            match running {
                Some((rtype, rimplicit))
                    if shift_and_clone(rtype, (0, 1)) == **typ && rimplicit == *implicit =>
                {
                    output.push_str(&format!(" {}", name));
                }
                Some((rtype, _)) => {
                    output.push_str(&format!(": {},", rtype.generate()));
                    if *implicit {
                        output.push_str(&format!("implicit {}", name));
                    } else {
                        output.push_str(&name.to_string());
                    }
                }
                None => {
                    if *implicit {
                        output.push_str("implicit ");
                    }
                    output.push_str(&format!("({}", &name.to_string()));
                }
            }
            running = Some((typ, *implicit));
        }

        if let Some((rtype, _)) = running {
            output.push_str(&format!(": {})", rtype.generate()));
        }
        output
    }
}

impl Generate for Exp {
    fn generate(&self) -> String {
        match self {
            Exp::Variable(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::TypCtor(e) => e.generate(),
            Exp::Call(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::DotCall(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::Anno(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::TypeUniv(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::LocalMatch(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::LocalComatch(e) => e.print_html_to_string(Some(&PrintCfg::default())),
            Exp::Hole(e) => e.print_html_to_string(Some(&PrintCfg::default())),
        }
    }
}

impl Generate for TypCtor {
    fn generate(&self) -> String {
        let TypCtor { span: _, name, args } = self;
        if name.id == "Fun" && args.len() == 2 {
            let arg = args.args[0].generate();
            let res = args.args[1].generate();
            format!("{} -> {}", arg, res)
        } else {
            format!(
                "{}{}",
                creat_page_link(&name.id),
                args.print_html_to_string(Some(&PrintCfg::default()))
            )
        }
    }
}

impl Generate for Args {
    fn generate(&self) -> String {
        if !self.args.iter().any(|x| !x.is_inserted_implicit()) {
            return String::new();
        }

        let mut output = String::new();
        output.push('(');
        let mut first = true;

        for arg in &self.args {
            if !arg.is_inserted_implicit() {
                if !first {
                    output.push_str(", ");
                }
                output.push_str(&arg.generate());
                first = false;
            }
        }

        output.push(')');
        output
    }
}

impl Generate for Arg {
    fn generate(&self) -> String {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.generate(),
            Arg::NamedArg { name: var, arg, .. } => {
                format!("{} := {}", var.id, arg.generate())
            }
            Arg::InsertedImplicitArg { .. } => {
                panic!("Inserted implicit arguments should not be generated")
            }
        }
    }
}

fn creat_page_link(name: &str) -> String {
    format!("<a href=\"#{}\">{}</a>", name, name)
}
