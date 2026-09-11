use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use linked_hash_map::LinkedHashMap;

use crate::config::ast::*;
use crate::config::visitor::VoidVisitor;
use crate::utils::error::{ErrorKind, MultiError};

pub struct InheritanceChecker<'err> {
    checked_node_parents: HashSet<*const RefCell<Dec>>,
    errors: &'err mut MultiError,
}

impl<'err> InheritanceChecker<'err> {
    fn new(errors: &'err mut MultiError) -> Self {
        Self {
            checked_node_parents: HashSet::new(),
            errors,
        }
    }

    pub fn do_pass<'a>(config: Config<'a>, errors: &mut MultiError) -> Config<'a> {
        InheritanceChecker::new(errors).visit_config(&config);
        config
    }

    fn check_no_circular_inheritance(
        &mut self,
        dec: &Rc<RefCell<Dec>>,
        name: &str,
        children: &LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> bool {
        if children.contains_key(&Rc::as_ptr(dec)) {
            let chain = children
                .iter()
                .map(|(_, d)| String::from(d.borrow().name()))
                .reduce(|acc, p| format!("{} -> {}", acc, p))
                .unwrap();
            self.errors.push(
                ErrorKind::Type,
                &format!(
                    "circular inheritance: `{}` is its own parent: `{} -> {}`",
                    name, chain, name
                ),
            );
            false
        } else {
            true
        }
    }

    fn check_declared_parent<'a>(
        &mut self,
        name: &str,
        parent: &'a Type,
    ) -> Option<(&'a str, Rc<RefCell<Dec>>)> {
        let Some(parent_dec) = &parent.dec else {
            self.errors.push(
                ErrorKind::Type,
                &format!(
                    "invalid parent in `{}`: `{}` is not a declared type",
                    name, parent.ty
                ),
            );
            return None;
        };
        let Ty::Name(parent_name) = &parent.ty else {
            self.errors
                .push(ErrorKind::Internal, "unexpected declared but unnamed type");
            return None;
        };
        Some((parent_name, parent_dec.upgrade().unwrap()))
    }

    fn check_duplicate_parent(
        &mut self,
        name: &str,
        parent_name: &str,
        parent_dec: &Rc<RefCell<Dec>>,
        all_parents: &HashSet<*const RefCell<Dec>>,
    ) {
        if all_parents.contains(&Rc::as_ptr(parent_dec)) {
            self.errors.push(
                ErrorKind::Type,
                &format!(
                    "invalid parent in `{}`: parent type `{}` is already present",
                    name, parent_name
                ),
            );
        }
    }

    fn check_single_parent_node(&mut self, name: &str, parent: &Option<String>) {
        if let Some(parent_name) = parent {
            self.errors.push(
                ErrorKind::Type,
                &format!(
                    "invalid parent in `{}`: already has a parent node `{}`",
                    name, parent_name
                ),
            );
        }
    }

    fn check_parent_is_node(
        &mut self,
        dec: &Rc<RefCell<Dec>>,
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> bool {
        if let Dec::Node { name, parents, .. } = &*dec.borrow() {
            self.check_node_parents(dec, name, parents, children);
            true
        } else {
            false
        }
    }

    fn check_node_parent(
        &mut self,
        name: &str,
        parent: &Type,
        recorded_parents: &mut HashSet<*const RefCell<Dec>>,
        parent_node: &mut Option<String>,
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) {
        let Some((parent_name, parent_dec)) = self.check_declared_parent(name, &parent) else {
            return;
        };
        self.check_duplicate_parent(name, parent_name, &parent_dec, recorded_parents);
        if self.check_parent_is_node(&parent_dec, children) {
            self.check_single_parent_node(name, parent_node);
            *parent_node = Some(parent_name.into());
        } else {
            self.errors.push(
                ErrorKind::Type,
                &format!(
                    "invalid parent in `{}`: `{}` is not a node",
                    name, parent_name
                ),
            );
        }
        recorded_parents.insert(Rc::as_ptr(&parent_dec));
    }

    fn check_node_parents(
        &mut self,
        dec: &Rc<RefCell<Dec>>,
        name: &str,
        parents: &[Box<Type>],
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) {
        if self.checked_node_parents.contains(&Rc::as_ptr(dec)) {
            return;
        }
        if !self.check_no_circular_inheritance(dec, name, children) {
            return;
        }
        children.insert(Rc::as_ptr(dec), Rc::clone(dec));
        let mut recorded_parents = HashSet::new();
        let mut parent_node = None;
        for parent in parents {
            self.check_node_parent(
                name,
                parent,
                &mut recorded_parents,
                &mut parent_node,
                children,
            );
        }
        children.remove(&Rc::as_ptr(dec));
        self.checked_node_parents.insert(Rc::as_ptr(dec));
    }
}

impl VoidVisitor for InheritanceChecker<'_> {
    fn visit_dec(&mut self, dec: &Rc<RefCell<Dec>>) {
        if let Dec::Node { name, parents, .. } = &*dec.borrow() {
            self.check_node_parents(dec, name, parents, &mut LinkedHashMap::new());
        }
        self.visit_dec_default(dec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_parent_declared_missing() {
        let mut errors = MultiError::new();
        let parent_type = Type::new(Ty::Name("UndeclaredParent".into()));
        let node_name = "MyNode".to_string();
        let result =
            InheritanceChecker::new(&mut errors).check_declared_parent(&node_name, &parent_type);
        assert!(result.is_none());
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));
        let err = errors.iter().next().unwrap();
        assert_eq!(err.error_kind(), &ErrorKind::Type);
        assert_eq!(
            err.message(),
            "invalid parent in `MyNode`: `UndeclaredParent` is not a declared type"
        );
    }

    #[test]
    fn test_check_duplicate_parent() {
        let mut errors = MultiError::new();
        let dummy_dec = Rc::new(RefCell::new(Dec::Enum {
            name: "Foo".into(),
            values: vec![],
        }));
        let mut recorded = HashSet::new();
        recorded.insert(Rc::as_ptr(&dummy_dec));

        let node_name = "MyNode".to_string();
        let parent_name = "Foo".to_string();
        InheritanceChecker::new(&mut errors).check_duplicate_parent(
            &node_name,
            &parent_name,
            &dummy_dec,
            &recorded,
        );
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));
        let err = errors.iter().next().unwrap();
        assert_eq!(err.error_kind(), &ErrorKind::Type);
        assert_eq!(
            err.message(),
            "invalid parent in `MyNode`: parent type `Foo` is already present"
        );
    }

    #[test]
    fn test_check_single_parent_node() {
        let mut errors = MultiError::new();
        let node_name = "MyNode".to_string();
        let parent_node = Some("ExistingParent".to_string());
        InheritanceChecker::new(&mut errors).check_single_parent_node(&node_name, &parent_node);
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));
        let err = errors.iter().next().unwrap();
        assert_eq!(err.error_kind(), &ErrorKind::Type);
        assert_eq!(
            err.message(),
            "invalid parent in `MyNode`: already has a parent node `ExistingParent`"
        );
    }

    #[test]
    fn test_check_parent_is_node() {
        let mut errors = MultiError::new();
        let node_dec = Rc::new(RefCell::new(Dec::Node {
            name: "SomeNode".into(),
            parents: vec![],
            decs: vec![],
        }));
        let mut decs = LinkedHashMap::new();

        let is_node =
            InheritanceChecker::new(&mut errors).check_parent_is_node(&node_dec, &mut decs);
        assert!(is_node);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_parent_is_not_node() {
        let mut errors = MultiError::new();
        let enum_dec = Rc::new(RefCell::new(Dec::Enum {
            name: "SomeEnum".into(),
            values: vec![],
        }));
        let mut decs = LinkedHashMap::new();

        let is_node =
            InheritanceChecker::new(&mut errors).check_parent_is_node(&enum_dec, &mut decs);
        assert!(!is_node);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_circular_inheritance() {
        let mut errors = MultiError::new();
        let node_dec = Rc::new(RefCell::new(Dec::Node {
            name: "A".into(),
            parents: vec![],
            decs: vec![],
        }));
        let mut decs = LinkedHashMap::new();
        decs.insert(Rc::as_ptr(&node_dec), Rc::clone(&node_dec));

        let node_name = "A".to_string();
        let ok = InheritanceChecker::new(&mut errors)
            .check_no_circular_inheritance(&node_dec, &node_name, &decs);
        assert!(!ok);
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));
        let err = errors.iter().next().unwrap();
        assert_eq!(err.error_kind(), &ErrorKind::Type);
        assert_eq!(
            err.message(),
            "circular inheritance: `A` is its own parent: `A -> A`"
        );
    }

    #[test]
    fn test_check_complex_circular_inheritance() {
        let mut errors = MultiError::new();
        let node_a = Rc::new(RefCell::new(Dec::Node {
            name: "A".into(),
            parents: vec![],
            decs: vec![],
        }));
        let node_b = Rc::new(RefCell::new(Dec::Node {
            name: "B".into(),
            parents: vec![],
            decs: vec![],
        }));
        let node_c = Rc::new(RefCell::new(Dec::Node {
            name: "C".into(),
            parents: vec![],
            decs: vec![],
        }));

        let mut decs = LinkedHashMap::new();
        decs.insert(Rc::as_ptr(&node_a), Rc::clone(&node_a));
        decs.insert(Rc::as_ptr(&node_b), Rc::clone(&node_b));
        decs.insert(Rc::as_ptr(&node_c), Rc::clone(&node_c));

        let node_name = "B".to_string();
        let ok = InheritanceChecker::new(&mut errors)
            .check_no_circular_inheritance(&node_b, &node_name, &decs);
        assert!(!ok);
        assert_eq!(errors.error_kind(), Some(ErrorKind::Type));
        let err = errors.iter().next().unwrap();
        assert_eq!(err.error_kind(), &ErrorKind::Type);
        assert_eq!(
            err.message(),
            "circular inheritance: `B` is its own parent: `A -> B -> C -> B`"
        );
    }
}
