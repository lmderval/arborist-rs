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
            .cloned()
            .and_then(|x| Some(Box::new(Type::new(Ty::Array(x)))));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::parser::ConfigParser;
    use crate::run_passes;

    use super::*;

    #[test]
    fn test_type_value_int() {
        let mut errors = MultiError::new();
        let mut resolver = ExpResolver::new(&mut errors);
        let mut val = Value::Int(42);
        let ty = resolver.type_value(&mut val);
        assert!(errors.is_empty());
        assert!(ty.dec.is_none());
        assert_eq!(ty.ty, Ty::Int(64));
    }

    #[test]
    fn test_type_value_str() {
        let mut errors = MultiError::new();
        let mut resolver = ExpResolver::new(&mut errors);
        let mut val = Value::Str("hello".to_string());
        let ty = resolver.type_value(&mut val);
        assert!(errors.is_empty());
        assert!(ty.dec.is_none());
        assert_eq!(ty.ty, Ty::Str);
    }

    #[test]
    fn test_type_value_array_int() {
        let mut errors = MultiError::new();
        let mut resolver = ExpResolver::new(&mut errors);
        let mut val = Value::Array(vec![
            Exp::new(Value::Int(1)),
            Exp::new(Value::Int(2)),
            Exp::new(Value::Int(3)),
        ]);
        let ty = resolver.type_value(&mut val);
        assert!(errors.is_empty());
        assert!(ty.dec.is_none());
        assert_eq!(ty.ty, Ty::Array(Box::new(Type::new(Ty::Int(64)))));
    }

    #[test]
    fn test_type_value_nested_array_str() {
        let mut errors = MultiError::new();
        let mut resolver = ExpResolver::new(&mut errors);
        let mut val = Value::Array(vec![
            Exp::new(Value::Array(vec![
                Exp::new(Value::Str("a".to_string())),
                Exp::new(Value::Str("b".to_string())),
            ])),
            Exp::new(Value::Array(vec![
                Exp::new(Value::Str("c".to_string())),
                Exp::new(Value::Str("d".to_string())),
            ])),
        ]);
        let ty = resolver.type_value(&mut val);
        assert!(errors.is_empty());
        assert!(ty.dec.is_none());
        assert_eq!(
            ty.ty,
            Ty::Array(Box::new(Type::new(Ty::Array(Box::new(Type::new(Ty::Str))))))
        );
    }

    #[test]
    fn test_check_int_expression() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
value = 42;
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => ExpResolver
        );
        assert!(errors.is_empty());
        let value = config.vars.get("value").unwrap();
        let value_ty = value.exp.ty.as_ref().unwrap();
        assert!(value_ty.dec.is_none());
        assert_eq!(value_ty.ty, Ty::Int(64));
    }

    #[test]
    fn test_check_str_expression() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
value = "hello";
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => ExpResolver
        );
        assert!(errors.is_empty());
        let value = config.vars.get("value").unwrap();
        let value_ty = value.exp.ty.as_ref().unwrap();
        assert!(value_ty.dec.is_none());
        assert_eq!(value_ty.ty, Ty::Str);
    }

    #[test]
    fn test_check_array_str_expression() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
value = [
    "hello",
    "world",
];
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => ExpResolver
        );
        assert!(errors.is_empty());
        let value = config.vars.get("value").unwrap();
        let value_ty = value.exp.ty.as_ref().unwrap();
        assert!(value_ty.dec.is_none());
        assert_eq!(value_ty.ty, Ty::Array(Box::new(Type::new(Ty::Str))));
        let Value::Array(inner_values) = &value.exp.value else {
            assert!(false);
            return;
        };
        for inner_value in inner_values {
            let inner_value_ty = inner_value.ty.as_ref().unwrap();
            assert!(inner_value_ty.dec.is_none());
            assert_eq!(inner_value_ty.ty, Ty::Str);
        }
    }

    #[test]
    fn test_check_nested_array_int_expression() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
value = [
    [0, 1],
    [1, 0],
];
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => ExpResolver
        );
        assert!(errors.is_empty());
        let value = config.vars.get("value").unwrap();
        let value_ty = value.exp.ty.as_ref().unwrap();
        assert!(value_ty.dec.is_none());
        assert_eq!(
            value_ty.ty,
            Ty::Array(Box::new(Type::new(Ty::Array(Box::new(Type::new(
                Ty::Int(64)
            ))))))
        );
        let Value::Array(inner0_exps) = &value.exp.value else {
            assert!(false);
            return;
        };
        for inner0_exp in inner0_exps {
            let inner0_exp_ty = inner0_exp.ty.as_ref().unwrap();
            assert!(inner0_exp_ty.dec.is_none());
            assert_eq!(
                inner0_exp_ty.ty,
                Ty::Array(Box::new(Type::new(Ty::Int(64))))
            );
            let Value::Array(inner1_exps) = &inner0_exp.value else {
                assert!(false);
                return;
            };
            for inner1_exp in inner1_exps {
                let inner1_exp_ty = inner1_exp.ty.as_ref().unwrap();
                assert!(inner1_exp_ty.dec.is_none());
                assert_eq!(inner1_exp_ty.ty, Ty::Int(64));
            }
        }
    }
}
