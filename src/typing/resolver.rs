use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::config::ast::*;
use crate::config::visitor::MutVoidVisitor;

pub struct Resolver {
    decs: HashMap<String, Weak<RefCell<Dec>>>,
}

impl Resolver {
    fn new(config: &Config) -> Self {
        let decs = config
            .decs
            .iter()
            .map(|dec| match &*dec.borrow() {
                Dec::Node { name, .. } | Dec::Enum { name, .. } => {
                    (name.clone(), Rc::downgrade(dec))
                }
                _ => panic!("unexpected declaration: should be a Node or an Enum"),
            })
            .collect();
        Self { decs }
    }

    pub fn resolve_config(config: &mut Config) {
        let mut resolver = Resolver::new(config);
        resolver.visit_config(config);
    }
}

impl MutVoidVisitor for Resolver {
    fn visit_type(&mut self, ty: &mut Rc<RefCell<Type>>) {
        let ty = &mut *ty.borrow_mut();
        match &mut ty.ty {
            Ty::Name(name) => {
                let Some(dec) = self.decs.get(name) else {
                    panic!("unkwnown type: {}", name);
                };
                ty.define(dec.clone());
            }
            Ty::Int(_) | Ty::Str | Ty::Loc => {}
            Ty::Array(ty) => self.visit_type(ty),
        }
    }
}
