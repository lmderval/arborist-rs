use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::ast::*;
use crate::config::visitor::MutVoidVisitor;
use crate::utils::error::{ErrorKind, MultiError};

pub struct NameResolver<'err> {
    decs: HashMap<String, Rc<RefCell<Dec>>>,
    errors: &'err mut MultiError,
}

impl<'err> NameResolver<'err> {
    fn new(config: &Config, errors: &'err mut MultiError) -> Self {
        let decs = config
            .decs
            .iter()
            .map(|dec| (dec.borrow().name().into(), Rc::clone(dec)))
            .collect();
        Self { decs, errors }
    }

    pub fn do_pass<'a>(mut config: Config<'a>, errors: &mut MultiError) -> Config<'a> {
        NameResolver::new(&config, errors).visit_config(&mut config);
        config
    }
}

impl MutVoidVisitor for NameResolver<'_> {
    fn visit_type(&mut self, ty: &mut Box<Type>) {
        match &mut ty.ty {
            Ty::Name(name) => {
                if let Some(dec) = self.decs.get(name) {
                    ty.define(Rc::downgrade(dec));
                } else {
                    self.errors
                        .push(ErrorKind::Type, &format!("no type named `{}`", name));
                };
            }
            Ty::Int(_) | Ty::Str | Ty::Loc => {}
            Ty::Array(ty) => self.visit_type(ty),
        }
    }

    fn visit_exp(&mut self, exp: &mut Exp) {
        self.visit_exp_default(exp);
        if let Some(ty) = &mut exp.ty {
            self.visit_type(ty);
        }
    }
}
