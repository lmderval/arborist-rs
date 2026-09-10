use crate::config::ast::*;
use crate::config::visitor::MutVoidVisitor;
use crate::utils::error::{ErrorKind, MultiError};

pub struct ExpResolver<'err> {
    ty: Option<Box<Type>>,
    errors: &'err mut MultiError,
}

impl<'err> ExpResolver<'err> {
    fn new(errors: &'err mut MultiError) -> Self {
        Self { ty: None, errors }
    }

    pub fn do_pass<'a>(mut config: Config<'a>, errors: &mut MultiError) -> Config<'a> {
        ExpResolver::new(errors).visit_config(&mut config);
        config
    }

    fn type_value(&mut self, value: &mut Value) -> Box<Type> {
        let old_ty = self.ty.take();
        self.visit_value(value);
        let ty = match self.ty.take() {
            Some(ty) => ty,
            None => {
                self.errors
                    .push(ErrorKind::Internal, "unexpected untyped value");
                Box::new(Type::new(ERROR_TY))
            }
        };
        self.ty = old_ty;
        ty
    }
}

impl MutVoidVisitor for ExpResolver<'_> {
    fn visit_exp(&mut self, exp: &mut Exp) {
        let ty = self.type_value(&mut exp.value);
        exp.type_exp(ty);
    }

    fn visit_int_val(&mut self, _val: &i64) {
        self.ty = Some(Box::new(Type::new(Ty::Int(64))));
    }

    fn visit_string_val(&mut self, _val: &mut String) {
        self.ty = Some(Box::new(Type::new(Ty::Str)));
    }

    fn visit_array_val(&mut self, val: &mut Vec<Exp>) {
        self.visit_array_val_default(val);
        self.ty = val
            .iter()
            .map(|e| e.ty.as_ref())
            .reduce(|res, ty| if res == ty { res } else { None })
            .and_then(|x| x)
            .cloned();
    }
}
