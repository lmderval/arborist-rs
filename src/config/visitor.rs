use crate::config::ast::*;

pub trait VoidVisitor {
    fn visit_config(&mut self, config: &Config) {
        config.vars.iter().for_each(|(_, v)| self.visit_var(v));
        config.decs.iter().for_each(|d| self.visit_dec(d));
    }

    fn visit_var(&mut self, var: &Var) {
        self.visit_exp(&var.exp);
    }

    fn visit_exp(&mut self, exp: &Exp) {
        self.visit_value(&exp.value);
    }

    fn visit_value(&mut self, _value: &Value) {}

    fn visit_dec(&mut self, dec: &Dec) {
        match &dec {
            Dec::Node {
                name,
                parents,
                decs,
            } => self.visit_node_dec(name, parents, decs),
            Dec::Enum { name, values } => self.visit_enum_dec(name, values),
            Dec::Attr { name, modifier, ty } => self.visit_attr_dec(name, modifier, ty),
        }
    }

    fn visit_node_dec(&mut self, _name: &String, parents: &Vec<Type>, decs: &Vec<Dec>) {
        parents.iter().for_each(|p| self.visit_type(p));
        decs.iter().for_each(|d| self.visit_dec(d));
    }

    fn visit_enum_dec(&mut self, _name: &String, _values: &Vec<String>) {}

    fn visit_attr_dec(&mut self, _name: &String, _modifier: &AttrMod, ty: &Type) {
        self.visit_type(ty);
    }

    fn visit_type(&mut self, ty: &Type) {
        match &ty.ty {
            Ty::Name(name) => self.visit_name_ty(name),
            Ty::Int(size) => self.visit_int_ty(size),
            Ty::Str => self.visit_str_ty(),
        }
    }

    fn visit_name_ty(&mut self, _name: &String) {}

    fn visit_int_ty(&mut self, _size: &u8) {}

    fn visit_str_ty(&mut self) {}
}
