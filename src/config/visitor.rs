use std::cell::RefCell;
use std::rc::Rc;

use crate::config::ast::*;

macro_rules! define_void_visitor {
    ( $name:ident, $iter:ident, $borrow:ident, ( $( $mut:tt )? ) ) => {
        pub trait $name {
            fn visit_config(&mut self, config: &$($mut)? Config) {
                self.visit_config_default(config);
            }

            fn visit_config_default(&mut self, config: &$($mut)? Config) {
                config.vars.$iter().for_each(|(_, v)| self.visit_var(v));
                config.decs.$iter().for_each(|d| self.visit_dec(d));
            }

            fn visit_var(&mut self, var: &$($mut)? Var) {
                self.visit_var_default(var);
            }

            fn visit_var_default(&mut self, var: &$($mut)? Var) {
                self.visit_exp(&$($mut)? var.exp);
            }

            fn visit_exp(&mut self, exp: &$($mut)? Exp) {
                self.visit_exp_default(exp);
            }

            fn visit_exp_default(&mut self, exp: &$($mut)? Exp) {
                self.visit_value(&$($mut)? exp.value);
            }

            fn visit_value(&mut self, value: &$($mut)? Value) {
                self.visit_value_default(value);
            }

            fn visit_value_default(&mut self, value: &$($mut)? Value) {
                match value {
                    Value::Int(n) => self.visit_int_val(n),
                    Value::Str(s) => self.visit_string_val(s),
                    Value::Array(v) => self.visit_array_val(v),
                }
            }

            fn visit_int_val(&mut self, val: &i64) {
                self.visit_int_val_default(val);
            }

            fn visit_int_val_default(&mut self, _val: &i64) {}

            fn visit_string_val(&mut self, val: &$($mut)? String) {
                self.visit_string_val_default(val);
            }

            fn visit_string_val_default(&mut self, _val: &$($mut)? String) {}

            fn visit_array_val(&mut self, val: &$($mut)? Vec<Exp>) {
                self.visit_array_val_default(val);
            }

            fn visit_array_val_default(&mut self, val: &$($mut)? Vec<Exp>) {
                val.$iter().for_each(|e| self.visit_exp(e));
            }

            fn visit_dec(&mut self, dec: &$($mut)? Rc<RefCell<Dec>>) {
                self.visit_dec_default(dec);
            }

            fn visit_dec_default(&mut self, dec: &$($mut)? Rc<RefCell<Dec>>) {
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
                name: &$($mut)? String,
                parents: &$($mut)? Vec<Box<Type>>,
                decs: &$($mut)? Vec<Rc<RefCell<Dec>>>,
            ) {
                self.visit_node_dec_default(name, parents, decs);
            }

            fn visit_node_dec_default(
                &mut self,
                _name: &$($mut)? String,
                parents: &$($mut)? Vec<Box<Type>>,
                decs: &$($mut)? Vec<Rc<RefCell<Dec>>>,
            ) {
                parents.$iter().for_each(|p| self.visit_type(p));
                decs.$iter().for_each(|d| self.visit_dec(d));
            }

            fn visit_enum_dec(
                &mut self,
                name: &$($mut)? String,
                values: &$($mut)? Vec<String>,
            ) {
                self.visit_enum_dec_default(name, values);
            }

            fn visit_enum_dec_default(
                &mut self,
                _name: &$($mut)? String,
                _values: &$($mut)? Vec<String>,
            ) {}

            fn visit_attr_dec(
                &mut self,
                name: &$($mut)? String,
                modifier: &$($mut)? AttrMod,
                ty: &$($mut)? Box<Type>,
            ) {
                self.visit_attr_dec_default(name, modifier, ty);
            }

            fn visit_attr_dec_default(
                &mut self,
                _name: &$($mut)? String,
                _modifier: &$($mut)? AttrMod,
                ty: &$($mut)? Box<Type>,
            ) {
                self.visit_type(ty);
            }

            fn visit_type(&mut self, ty: &$($mut)? Box<Type>) {
                self.visit_type_default(ty);
            }

            fn visit_type_default(&mut self, ty: &$($mut)? Box<Type>) {
                match &$($mut)? ty.ty {
                    Ty::Name(name) => self.visit_name_ty(name),
                    Ty::Int(size) => self.visit_int_ty(size),
                    Ty::Str => self.visit_str_ty(),
                    Ty::Loc => self.visit_loc_ty(),
                    Ty::Array(ty) => self.visit_array_ty(ty),
                    Ty::Vector(ty) => self.visit_vector_ty(ty),
                    Ty::Any => self.visit_any_ty(),
                }
            }

            fn visit_name_ty(&mut self, name: &$($mut)? String) {
                self.visit_name_ty_default(name);
            }

            fn visit_name_ty_default(&mut self, _name: &$($mut)? String) {}

            fn visit_int_ty(&mut self, size: &u8) {
                self.visit_int_ty_default(size);
            }

            fn visit_int_ty_default(&mut self, _size: &u8) {}

            fn visit_str_ty(&mut self) {
                self.visit_str_ty_default();
            }

            fn visit_str_ty_default(&mut self) {}

            fn visit_loc_ty(&mut self) {
                self.visit_loc_ty_default();
            }

            fn visit_loc_ty_default(&mut self) {}

            fn visit_array_ty(&mut self, ty: &$($mut)? Box<Type>) {
                self.visit_array_ty_default(ty);
            }

            fn visit_array_ty_default(&mut self, ty: &$($mut)? Box<Type>) {
                self.visit_type(ty);
            }

            fn visit_vector_ty(&mut self, ty: &$($mut)? Box<Type>) {
                self.visit_vector_ty_default(ty);
            }

            fn visit_vector_ty_default(&mut self, ty: &$($mut)? Box<Type>) {
                self.visit_type(ty);
            }

            fn visit_any_ty(&mut self) {
                self.visit_any_ty_default();
            }

            fn visit_any_ty_default(&mut self) {}
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
