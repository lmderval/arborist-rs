use crate::config::ast::*;
use crate::config::visitor::MutVoidVisitor;
use crate::utils::error::MultiError;

pub struct ExpResolver {
    ty: Option<Box<Type>>,
}

impl ExpResolver {
    fn new() -> Self {
        Self { ty: None }
    }

    pub fn do_pass<'a>(mut config: Config<'a>, _errors: &mut MultiError) -> Config<'a> {
        Self::new().visit_config(&mut config);
        config
    }

    fn type_value(&mut self, value: &mut Value) -> Result<Box<Type>, String> {
        let old_ty = self.ty.take();
        self.visit_value(value);
        let Some(ty) = self.ty.take() else {
            return Err("unexpected untyped value".into());
        };
        self.ty = old_ty;
        Ok(ty)
    }
}

impl MutVoidVisitor for ExpResolver {
    fn visit_exp(&mut self, exp: &mut Exp) {
        match self.type_value(&mut exp.value) {
            Ok(ty) => exp.type_exp(ty),
            Err(err) => panic!("{}", err),
        }
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
