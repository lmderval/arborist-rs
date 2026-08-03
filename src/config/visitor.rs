use crate::config::ast::*;

pub trait VoidVisitor {
    fn visit_config(&mut self, config: &Config) {
        config.decs.iter().for_each(|d| self.visit_dec(d));
    }

    fn visit_dec(&mut self, dec: &Dec) {
        match &dec {
            Dec::Node { name, parents } => self.visit_node_dec(name, parents),
            Dec::Enum { name, values } => self.visit_enum_dec(name, values),
        }
    }

    fn visit_node_dec(&mut self, _name: &String, parents: &Vec<Type>) {
        parents.iter().for_each(|p| self.visit_type(p));
    }

    fn visit_enum_dec(&mut self, _name: &String, _values: &Vec<String>) {}

    fn visit_type(&mut self, ty: &Type) {
        match &ty.ty {
            Ty::Name(name) => self.visit_name_ty(name),
        }
    }

    fn visit_name_ty(&mut self, _name: &String) {}
}
