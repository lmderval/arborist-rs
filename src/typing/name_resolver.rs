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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;

    use super::*;
    use crate::config::parser::ConfigParser;
    use crate::run_passes;

    #[test]
    fn test_resolve_node_parents() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Base {}
node Derived (Base) {}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 2);

        let base_dec = &config.decs[0];
        let derived_dec = &config.decs[1];

        let Dec::Node {
            name: base_name, ..
        } = &*base_dec.borrow()
        else {
            panic!("expected Base to be a node");
        };
        assert_eq!(base_name, "Base");

        let Dec::Node {
            name: derived_name,
            parents,
            ..
        } = &*derived_dec.borrow()
        else {
            panic!("expected Derived to be a node");
        };
        assert_eq!(derived_name, "Derived");
        assert_eq!(parents.len(), 1);

        let parent_type = &parents[0];
        let Ty::Name(parent_name) = &parent_type.ty else {
            panic!("expected Name type");
        };
        assert_eq!(parent_name, "Base");
        assert!(parent_type.dec.is_some());
        assert!(Rc::ptr_eq(
            &parent_type.dec.as_ref().unwrap().upgrade().unwrap(),
            base_dec
        ));
    }

    #[test]
    fn test_resolve_node_attribute_enum() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
enum Kind {
    FIRST,
    SECOND
}

node Container {
    kind: Kind;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 2);

        let kind_enum_dec = &config.decs[0];
        let container_node_dec = &config.decs[1];

        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: kind_name,
            ty: kind_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(kind_name, "kind");
        let Ty::Name(kind_ty_name) = &kind_type.ty else {
            panic!("expected Name type");
        };
        assert_eq!(kind_ty_name, "Kind");
        assert!(Rc::ptr_eq(
            &kind_type.dec.as_ref().unwrap().upgrade().unwrap(),
            kind_enum_dec
        ));
    }

    #[test]
    fn test_resolve_node_attribute_node() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Child {}

node Container {
    child: Child;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 2);

        let child_node_dec = &config.decs[0];
        let container_node_dec = &config.decs[1];

        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: child_name,
            ty: child_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(child_name, "child");
        let Ty::Name(child_ty_name) = &child_type.ty else {
            panic!("expected Name type");
        };
        assert_eq!(child_ty_name, "Child");
        assert!(Rc::ptr_eq(
            &child_type.dec.as_ref().unwrap().upgrade().unwrap(),
            child_node_dec
        ));
    }

    #[test]
    fn test_resolve_node_attribute_array_enum() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
enum Kind {
    FIRST,
    SECOND
}

node Container {
    kinds: arr<Kind>;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 2);

        let kind_enum_dec = &config.decs[0];
        let container_node_dec = &config.decs[1];

        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: kinds_name,
            ty: kinds_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(kinds_name, "kinds");
        assert!(kinds_type.dec.is_none());
        let Ty::Array(inner_kind_type) = &kinds_type.ty else {
            panic!("expected Array type");
        };
        let Ty::Name(inner_kind_name) = &inner_kind_type.ty else {
            panic!("expected Name type");
        };
        assert_eq!(inner_kind_name, "Kind");
        assert!(Rc::ptr_eq(
            &inner_kind_type.dec.as_ref().unwrap().upgrade().unwrap(),
            kind_enum_dec
        ));
    }

    #[test]
    fn test_resolve_node_attribute_array_node() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Child {}

node Container {
    children: arr<Child>;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 2);

        let child_node_dec = &config.decs[0];
        let container_node_dec = &config.decs[1];

        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: children_name,
            ty: children_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(children_name, "children");
        assert!(children_type.dec.is_none());
        let Ty::Array(inner_child_type) = &children_type.ty else {
            panic!("expected Array type");
        };
        let Ty::Name(inner_child_name) = &inner_child_type.ty else {
            panic!("expected Name type");
        };
        assert_eq!(inner_child_name, "Child");
        assert!(Rc::ptr_eq(
            &inner_child_type.dec.as_ref().unwrap().upgrade().unwrap(),
            child_node_dec
        ));
    }

    #[test]
    fn test_resolve_node_attribute_primitive_int() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Container {
    count: i32;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 1);

        let container_node_dec = &config.decs[0];
        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: count_name,
            ty: count_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(count_name, "count");
        assert_eq!(count_type.ty, Ty::Int(32));
        assert!(count_type.dec.is_none());
    }

    #[test]
    fn test_resolve_node_attribute_primitive_str() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Container {
    tag: str;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 1);

        let container_node_dec = &config.decs[0];
        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: tag_name,
            ty: tag_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(tag_name, "tag");
        assert_eq!(tag_type.ty, Ty::Str);
        assert!(tag_type.dec.is_none());
    }

    #[test]
    fn test_resolve_node_attribute_primitive_loc() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Container {
    location: loc;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 1);

        let container_node_dec = &config.decs[0];
        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: loc_name,
            ty: loc_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(loc_name, "location");
        assert_eq!(loc_type.ty, Ty::Loc);
        assert!(loc_type.dec.is_none());
    }

    #[test]
    fn test_resolve_node_attribute_array_primitive() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node Container {
    numbers: arr<i64>;
}
"#;
        let config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert!(errors.is_empty());
        assert_eq!(config.decs.len(), 1);

        let container_node_dec = &config.decs[0];
        let Dec::Node {
            decs: attr_decs, ..
        } = &*container_node_dec.borrow()
        else {
            panic!("expected Container to be a node");
        };
        assert_eq!(attr_decs.len(), 1);

        let Dec::Attr {
            name: numbers_name,
            ty: numbers_type,
            ..
        } = &*attr_decs[0].borrow()
        else {
            panic!("expected Attr");
        };
        assert_eq!(numbers_name, "numbers");
        assert!(numbers_type.dec.is_none());
        let Ty::Array(inner_int_type) = &numbers_type.ty else {
            panic!("expected Array type");
        };
        assert_eq!(inner_int_type.ty, Ty::Int(64));
        assert!(inner_int_type.dec.is_none());
    }

    #[test]
    fn test_resolve_unknown_type_error() {
        let mut errors = MultiError::new();
        let file = PathBuf::new();
        let source = r#"
node BadNode (UnknownParent) {
    attr: UnknownType;
}
"#;
        let _config = run_passes!(
            (source.into(), errors)
            => ConfigParser(&file)
            => NameResolver
        );
        assert_eq!(errors.count(), 2);
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));

        let mut error_messages: Vec<&str> = errors.iter().map(|e| e.message()).collect();
        error_messages.sort();
        assert_eq!(
            error_messages,
            vec![
                "no type named `UnknownParent`",
                "no type named `UnknownType`"
            ]
        );
    }
}
