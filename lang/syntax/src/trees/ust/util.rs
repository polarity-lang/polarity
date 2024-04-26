use super::def::*;

pub trait Instantiate {
    fn instantiate(&self) -> TelescopeInst;
}

impl Instantiate for Telescope {
    fn instantiate(&self) -> TelescopeInst {
        let params = self
            .params
            .iter()
            .map(|Param { name, .. }| ParamInst {
                span: None,
                name: name.clone(),
                info: (),
                typ: (),
            })
            .collect();
        TelescopeInst { params }
    }
}
