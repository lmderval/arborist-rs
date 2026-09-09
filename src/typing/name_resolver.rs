use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::ast::*;
use crate::config::visitor::MutVoidVisitor;
use crate::utils::error::MultiError;

pub struct NameResolver {
    decs: HashMap<String, Rc<RefCell<Dec>>>,
}

impl NameResolver {
    fn new(config: &Config) -> Self {
        let decs = config
            .decs
            .iter()
            .map(|dec| match &*dec.borrow() {
                Dec::Node { name, .. } | Dec::Enum { name, .. } => (name.clone(), Rc::clone(dec)),
                _ => panic!("unexpected declaration: should be a Node or an Enum"),
            })
            .collect();
        Self { decs }
    }

    pub fn do_pass<'a>(mut config: Config<'a>, _errors: &mut MultiError) -> Config<'a> {
        Self::new(&config).visit_config(&mut config);
        config
    }
}

impl MutVoidVisitor for NameResolver {
    fn visit_type(&mut self, ty: &mut Box<Type>) {
        match &mut ty.ty {
            Ty::Name(name) => {
                let Some(dec) = self.decs.get(name) else {
                    panic!("unkwnown type: {}", name);
                };
                ty.define(Rc::downgrade(dec));
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
