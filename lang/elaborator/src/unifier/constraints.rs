use std::rc::Rc;

use printer::Print;
use syntax::ast::Exp;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Constraint {
    pub lhs: Rc<Exp>,
    pub rhs: Rc<Exp>,
}

impl Print for Constraint {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        self.lhs.print(cfg, alloc).append(" = ").append(self.rhs.print(cfg, alloc))
    }
}

