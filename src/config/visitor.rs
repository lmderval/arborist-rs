use std::cell::RefCell;
use std::rc::Rc;

use crate::config::ast::*;

macro_rules! define_void_visitor {
    ( $name:ident, $iter:ident, $borrow:ident, ( $( $mut:tt )? ) ) => {
        pub trait $name {
            fn visit_config(&mut self, config: &$($mut)? Config) {
                config.vars.$iter().for_each(|(_, v)| self.visit_var(v));
                config.decs.$iter().for_each(|d| self.visit_dec(d));
            }

            fn visit_var(&mut self, var: &$($mut)? Var) {
                self.visit_exp(&$($mut)? var.exp);
            }

            fn visit_exp(&mut self, exp: &$($mut)? Exp) {
                self.visit_value(&$($mut)? exp.value);
            }

            fn visit_value(&mut self, value: &$($mut)? Value) {
                match value {
                    Value::Int(n) => self.visit_int_val(n),
                    Value::Str(s) => self.visit_string_val(s),
                    Value::Array(v) => self.visit_array_val(v),
                }
            }

            fn visit_int_val(&mut self, _val: &i64) {}

            fn visit_string_val(&mut self, _val: &$($mut)? String) {}

            fn visit_array_val(&mut self, val: &$($mut)? Vec<Exp>) {
                val.$iter().for_each(|e| self.visit_exp(e));
            }

            fn visit_dec(&mut self, dec: &$($mut)? Rc<RefCell<Dec>>) {
                match &$($mut)? *dec.$borrow() {
                    Dec::Node {
                        name,
                        parents,
                        decs,
                    } => self.visit_node_dec(name, parents, decs),
                    Dec::Enum {
                        name,
                        values,
                    } => self.visit_enum_dec(name, values),
                    Dec::Attr {
                        name,
                        modifier,
                        ty,
                    } => self.visit_attr_dec(name, modifier, ty),
                }
            }

            fn visit_node_dec(
                &mut self,
                _name: &$($mut)? String,
                parents: &$($mut)? Vec<Rc<RefCell<Type>>>,
                decs: &$($mut)? Vec<Rc<RefCell<Dec>>>,
            ) {
                parents.$iter().for_each(|p| self.visit_type(p));
                decs.$iter().for_each(|d| self.visit_dec(d));
            }

            fn visit_enum_dec(
                &mut self,
                _name: &$($mut)? String,
                _values: &$($mut)? Vec<String>,
            ) {}

            fn visit_attr_dec(
                &mut self,
                _name: &$($mut)? String,
                _modifier: &$($mut)? AttrMod,
                ty: &$($mut)? Rc<RefCell<Type>>,
            ) {
                self.visit_type(ty);
            }

            fn visit_type(&mut self, ty: &$($mut)? Rc<RefCell<Type>>) {
                match &$($mut)? ty.$borrow().ty {
                    Ty::Name(name) => self.visit_name_ty(name),
                    Ty::Int(size) => self.visit_int_ty(size),
                    Ty::Str => self.visit_str_ty(),
                    Ty::Array(ty) => self.visit_array_ty(ty),
                }
            }

            fn visit_name_ty(&mut self, _name: &$($mut)? String) {}

            fn visit_int_ty(&mut self, _size: &u8) {}

            fn visit_str_ty(&mut self) {}

            fn visit_array_ty(&mut self, ty: &$($mut)? Rc<RefCell<Type>>) {
                self.visit_type(ty);
            }
        }
    };

    ( VoidVisitor ) => {
        define_void_visitor!(VoidVisitor, iter, borrow, ());
    };

    ( MutVoidVisitor ) => {
        define_void_visitor!(MutVoidVisitor, iter_mut, borrow_mut, (mut));
    };
}

define_void_visitor!(VoidVisitor);

define_void_visitor!(MutVoidVisitor);
