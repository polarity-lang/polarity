use std::rc::Rc;

use crate::de_bruijn::*;

use super::def::*;

pub struct Assign<K, V>(pub K, pub V);

pub trait Substitution {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>>;
}

impl<T: AsRef<[Rc<Exp>]>> Substitution for T {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>> {
        if lvl.fst != 0 {
            return None;
        }
        Some(self.as_ref()[lvl.snd].clone())
    }
}

impl Substitution for Assign<Lvl, &Rc<Exp>> {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable: Sized {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self;
}

pub trait SubstTelescope {
    fn subst_telescope<S: Substitution>(&self, lvl: Lvl, by: &S) -> Self;
}

impl Substitutable for Rc<Exp> {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self {
        match &**self {
            Exp::Var { info, idx } => match by.get(lvl.relative(*idx)) {
                Some(exp) => exp,
                None => Rc::new(Exp::Var { info: info.clone(), idx: *idx }),
            },
            Exp::TypCtor { info, name, args: subst } => Rc::new(Exp::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: subst.subst(lvl, by),
            }),
            Exp::Ctor { info, name, args: subst } => Rc::new(Exp::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: subst.subst(lvl, by),
            }),
            Exp::Dtor { info, exp, name, args: subst } => Rc::new(Exp::Dtor {
                info: info.clone(),
                exp: exp.subst(lvl, by),
                name: name.clone(),
                args: subst.subst(lvl, by),
            }),
            Exp::Anno { info, exp, typ } => Rc::new(Exp::Anno {
                info: info.clone(),
                exp: exp.subst(lvl, by),
                typ: typ.subst(lvl, by),
            }),
            Exp::Type { info } => Rc::new(Exp::Type { info: info.clone() }),
        }
    }
}

impl Substitutable for TypApp {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self {
        let TypApp { info, name, args: subst } = self;
        TypApp { info: info.clone(), name: name.clone(), args: subst.subst(lvl, by) }
    }
}

impl Substitutable for Args {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self {
        self.iter().map(|x| x.subst(lvl, by)).collect()
    }
}

impl Substitutable for Param {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(lvl, by) }
    }
}
