use ast::{Anno, Arg, Args, Attribute, Attributes, Call, Case, Ctor, DocComment, DotCall, Dtor, Exp, Hole, LocalComatch, LocalMatch, Motive, Param, ParamInst, Pattern, SelfParam, Telescope, TelescopeInst, TypCtor, TypeUniv, Variable};

pub trait Generate{
    fn generate(&self) -> String;
}
impl Generate for Ctor{
    fn generate(&self) -> String {
        let Ctor { span: _, doc, name, params, typ } = self;

        let doc_str = doc.generate();
        let head = format!("{}{}", name.id, params.generate());

        let head = if typ.is_simple() {
            head
        } else {
            format!("{}: {}", head, typ.generate())
        };

        format!("{}{}", doc_str, head)
    }
}

impl Generate for Dtor{
    fn generate(&self) -> String {
        let Dtor { span: _, doc, name, params, self_param, ret_typ } = self;

        let doc_str = doc.generate();
        let head = if self_param.is_simple(){
            ".".to_owned()
        } else {
            format!("{}.", self_param.generate())
        };
        
        format!("{}{}{}{}: {}",doc_str, head, name.id, params.generate(), ret_typ.generate())
    }
}

impl Generate for Telescope{
    fn generate(&self) -> String {
        let Telescope { params } = self;
        let mut output = String::new();
        if params.is_empty() {
            return output;
        }
        let mut running: Option<(&Exp, bool)> = None;
        for Param { implicit, name, typ } in params {
            match running {
                // We need to shift before comparing to ensure we compare the correct De-Bruijn indices
                Some((rtype, rimplicit)) if rtype == typ.as_ref() && rimplicit == *implicit => {
                    // We are adding another parameter of the same type.
                    output.push_str(&format!(" {} ", name.id));
                }
                Some((rtype, _)) => {
                    // We are adding another parameter with a different type,
                    // and have to close the previous list first.
                    output.push_str(&format!(" : {}, ", rtype.generate()));
                    if *implicit {
                        output.push_str(&format!("implicit {} ", name.id));
                    } else {
                        output.push_str(&name.id);
                    }
                }
                None => {
                    // We are starting a new chunk and adding the very first parameter.
                    // If we are starting a chunk of implicit parameters then we also have to
                    // add the "implicit" keyword at this point.
                    if *implicit {
                        output.push_str("implicit ");
                    }
                    output.push_str(&name.id);
                }
            }
            running = Some((typ, *implicit));
        }

        // Close the last parameter
        if let Some((rtype, _)) = running {
            output.push_str(&format!(" : {}", rtype.generate()));
        }

       "(".to_owned() + &output + &")".to_owned()
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

impl<T: Generate> Generate for Vec<T> {
    fn generate(&self) -> String {
        self.iter().map(|value| value.generate()).collect::<Vec<String>>().join(",<br>")
    }
}

impl<T: Generate> Generate for Box<T> {
    fn generate(&self) -> String {
        self.as_ref().generate()
    }
}

impl Generate for SelfParam{
    fn generate(&self) -> String {
        let SelfParam { info: _, name, typ } = self;

        match name {
            Some(name) => format!("{} : {}", name.id, typ.generate()),
            None => typ.generate(),
        }
    }}
impl Generate for DocComment {
    fn generate(&self) -> String {
        let DocComment { docs } = self;
        let prefix = "<span class=\"comment\"> -- |";
        let posfix = "</span>";
        docs.iter().map(|doc| format!("{} {} {}", prefix, doc, posfix)).collect::<Vec<String>>().join("<br>") + &"<br>".to_owned()
    }
}

impl Generate for Attribute{
    fn generate(&self) -> String {
        match self {
            Attribute::OmitPrint => "omit_print".to_owned(),
            Attribute::Opaque => "opaque".to_owned(),
            Attribute::Transparent => "transparent".to_owned(),
            Attribute::Other(s) => s.to_owned(),
        }
    }
}

impl Generate for Attributes{
    fn generate(&self) -> String {
        if self.attrs.is_empty() {
            "".to_owned()
        } else {
            "#".to_owned() + "( " + &self.attrs.iter().map(|attr| attr.generate()).collect::<Vec<String>>().join(", ") + " )"
        }
    }
}





impl Generate for Param{
    fn generate(&self) -> String {
        let Param { implicit, name, typ } = self;
        if *implicit {
            format!("implicit {} : {}",name.id, typ.generate())
        } else {
            format!("{} : ({})",name.id, typ.generate())
        }
    }
}

impl Generate for Exp{
    fn generate(&self) -> String {
        match self {
            Exp::Variable(variable) => variable.generate(),
            Exp::TypCtor(typ_ctor) => typ_ctor.generate(),
            Exp::Call(call) => call.generate(),
            Exp::DotCall(dot_call) => dot_call.generate(),
            Exp::Anno(anno) => anno.generate(),
            Exp::TypeUniv(type_univ) => type_univ.generate(),
            Exp::LocalMatch(local_match) => local_match.generate(),
            Exp::LocalComatch(local_comatch) => local_comatch.generate(),
            Exp::Hole(hole) => hole.generate(),
        }
    }
}

impl Generate for Variable{
    fn generate(&self) -> String {
        self.name.id.clone()
    }
}

impl Generate for TypCtor{
    fn generate(&self) -> String {
        let TypCtor { name, args, .. } = self;
        if name.id == "Fun" && args.len() == 2 {
            let arg = args.args[0].generate();
            let res = args.args[1].generate();
            format!("{} -> {}", arg, res)
        } else if !args.is_empty() {
            format!("{}({})", name.id, args.generate())    
        } else {
            name.id.clone()
        }
    }
}

impl Generate for Call{
    fn generate(&self) -> String {
        let Call { name, args, .. } = self;
        format!("{}{}", name.id, args.generate())
    }
}

impl Generate for DotCall{
    fn generate(&self) -> String {
        let DotCall { exp, name, args, .. } = self;
        let mut result = format!("{}.{}", exp.generate(), name.id);
        if !args.args.is_empty() {
            result = format!("{}({})", result, args.generate());
        }
        result
    }
}

impl Generate for Anno{
    fn generate(&self) -> String {
    let Anno { exp, typ, .. } = self;
    format!("{} : {}", exp.generate(), typ.generate())
    }
}

impl Generate for TypeUniv{
    fn generate(&self) -> String {
        "Type".to_string()
    }
}

impl Generate for LocalMatch{
    fn generate(&self) -> String {
        let LocalMatch { name, on_exp, motive, cases, .. } = self;
        format!(
            "{}.match {} {} {}",
            on_exp.generate(),
            name.user_name.as_ref().map_or("".to_string(), |n| n.id.clone()),
            motive.as_ref().map_or("".to_string(), |m| m.generate()),
            cases.generate()
        )
    }
}

impl Generate for LocalComatch{
    fn generate(&self) -> String {
        let LocalComatch { name, is_lambda_sugar, cases, .. } = self;
        if *is_lambda_sugar {
            format!("lambda_sugar({})", cases.generate())
        } else {
            format!(
                "comatch {} {}",
                name.user_name.as_ref().map_or("".to_string(), |n| n.id.clone()),
                cases.generate()
            )
        }
    }
}

impl Generate for Arg{
    fn generate(&self) -> String {
        match self {
            Arg::NamedArg(var_bound,exp ) 
            => format!("{}({})",
                var_bound.id.clone(),
                exp.generate()),
            Arg::UnnamedArg(exp) => exp.generate(),
            Arg::InsertedImplicitArg(_) => "Hole".to_string(),
            
        }
    }
}

impl Generate for Args{
    fn generate(&self) -> String {
        self.args.iter().map(|arg| arg.generate()).collect::<Vec<String>>().join(", ")
    }
}

impl Generate for Case{
    fn generate(&self) -> String {
    let Case { pattern, body, .. } = self;

    let body_str = match body {
        None => "absurd".to_string(),
        Some(body) => format!("=> {}", body.generate()),
    };

    format!("{} {}", pattern.generate(), body_str)
    }
}


impl Generate for Pattern{
    fn generate(&self) -> String {
        let Pattern { is_copattern, name, params } = self;
        let copattern = if *is_copattern { "co" } else { "" };
        format!("{}{} {}", copattern, name.id, params.generate())
    }
}

impl Generate for TelescopeInst{
    fn generate(&self) -> String {
        let TelescopeInst { params } = self;
        format!("{}", params.generate())
    }
}

impl Generate for ParamInst{
    fn generate(&self) -> String {
        let ParamInst { span: _, info: _, name, typ: _ } = self;
        name.id.clone()
    }
}

impl Generate for Motive{
    fn generate(&self) -> String {
        let Motive { param, ret_typ, .. } = self;
        format!("{} => {}", param.generate(), ret_typ.generate())
    }
}

impl Generate for Hole{
    fn generate(&self) -> String {
        "_".to_string()
    }
}