use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use linked_hash_map::LinkedHashMap;

use crate::config::ast::*;
use crate::config::visitor::VoidVisitor;

pub struct TypeChecker {
    checked_node_parents: HashSet<*const RefCell<Dec>>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            checked_node_parents: HashSet::new(),
        }
    }

    pub fn do_pass(config: Config) -> Config {
        Self::new().visit_config(&config);
        config
    }

    fn check_no_circular_inheritance(
        &self,
        dec: &Rc<RefCell<Dec>>,
        name: &str,
        children: &LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> Result<(), String> {
        if children.contains_key(&Rc::as_ptr(dec)) {
            let chain = children
                .iter()
                .map(|(_, d)| String::from(d.borrow().name()))
                .reduce(|acc, p| format!("{} -> {}", acc, p))
                .unwrap();
            return Err(format!(
                "circular inheritance: `{}` is its own parent: `{} -> {}`",
                name, chain, name
            ));
        }
        Ok(())
    }

    fn check_declared_parent<'a>(
        &self,
        name: &str,
        parent: &'a Type,
    ) -> Result<(&'a str, Rc<RefCell<Dec>>), String> {
        let Some(parent_dec) = &parent.dec else {
            return Err(format!(
                "invalid parent in `{}`: `{}` is not a declared type",
                name, parent.ty
            ));
        };
        let Ty::Name(parent_name) = &parent.ty else {
            unreachable!("unexpected declared but unnamed type");
        };
        Ok((parent_name, parent_dec.upgrade().unwrap()))
    }

    fn check_duplicate_parent(
        &self,
        name: &str,
        parent_name: &str,
        parent_dec: &Rc<RefCell<Dec>>,
        all_parents: &HashSet<*const RefCell<Dec>>,
    ) -> Result<(), String> {
        if all_parents.contains(&Rc::as_ptr(parent_dec)) {
            return Err(format!(
                "invalid parent in `{}`: parent type `{}` is already present",
                name, parent_name
            ));
        }
        Ok(())
    }

    fn check_single_parent_node(&self, name: &str, parent: &Option<String>) -> Result<(), String> {
        if let Some(parent_name) = parent {
            return Err(format!(
                "invalid parent in `{}`: already has a parent node `{}`",
                name, parent_name
            ));
        }
        Ok(())
    }

    fn check_parent_is_node(
        &mut self,
        dec: &Rc<RefCell<Dec>>,
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> Result<bool, String> {
        if let Dec::Node { name, parents, .. } = &*dec.borrow() {
            self.check_node_parents(dec, name, parents, children)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn check_node_parent(
        &mut self,
        name: &str,
        parent: &Type,
        recorded_parents: &mut HashSet<*const RefCell<Dec>>,
        parent_node: &mut Option<String>,
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> Result<(), String> {
        let (parent_name, parent_dec) = self.check_declared_parent(name, &parent)?;
        self.check_duplicate_parent(name, parent_name, &parent_dec, recorded_parents)?;
        if self.check_parent_is_node(&parent_dec, children)? {
            self.check_single_parent_node(name, parent_node)?;
            *parent_node = Some(parent_name.into());
        } else {
            return Err(format!(
                "invalid parent in `{}`: `{}` is not a node",
                name, parent_name
            ));
        }
        recorded_parents.insert(Rc::as_ptr(&parent_dec));
        Ok(())
    }

    fn check_node_parents(
        &mut self,
        dec: &Rc<RefCell<Dec>>,
        name: &str,
        parents: &[Box<Type>],
        children: &mut LinkedHashMap<*const RefCell<Dec>, Rc<RefCell<Dec>>>,
    ) -> Result<(), String> {
        if self.checked_node_parents.contains(&Rc::as_ptr(dec)) {
            return Ok(());
        }
        self.check_no_circular_inheritance(dec, name, children)?;
        children.insert(Rc::as_ptr(dec), Rc::clone(dec));
        let result = (|| {
            let mut recorded_parents = HashSet::new();
            let mut parent_node = None;
            for parent in parents {
                self.check_node_parent(
                    name,
                    parent,
                    &mut recorded_parents,
                    &mut parent_node,
                    children,
                )?;
            }
            Ok(())
        })();
        children.remove(&Rc::as_ptr(dec));
        self.checked_node_parents.insert(Rc::as_ptr(dec));
        result
    }
}

impl VoidVisitor for TypeChecker {
    fn visit_dec(&mut self, dec: &Rc<RefCell<Dec>>) {
        if let Dec::Node { name, parents, .. } = &*dec.borrow() {
            if let Err(err) = self.check_node_parents(dec, name, parents, &mut LinkedHashMap::new())
            {
                panic!("{}", err);
            }
        }
        self.visit_dec_default(dec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_parent_declared_missing() {
        let parent_type = Type::new(Ty::Name("UndeclaredParent".into()));
        let node_name = "MyNode".to_string();
        let result = TypeChecker::new().check_declared_parent(&node_name, &parent_type);
        match result {
            Err(err) => assert_eq!(
                err,
                "invalid parent in `MyNode`: `UndeclaredParent` is not a declared type"
            ),
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn test_check_duplicate_parent() {
        let dummy_dec = Rc::new(RefCell::new(Dec::Enum {
            name: "Foo".into(),
            values: vec![],
        }));
        let mut recorded = HashSet::new();
        recorded.insert(Rc::as_ptr(&dummy_dec));

        let node_name = "MyNode".to_string();
        let parent_name = "Foo".to_string();
        let result = TypeChecker::new().check_duplicate_parent(
            &node_name,
            &parent_name,
            &dummy_dec,
            &recorded,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "invalid parent in `MyNode`: parent type `Foo` is already present"
        );
    }

    #[test]
    fn test_check_single_parent_node() {
        let node_name = "MyNode".to_string();
        let parent_node = Some("ExistingParent".to_string());
        let result = TypeChecker::new().check_single_parent_node(&node_name, &parent_node);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "invalid parent in `MyNode`: already has a parent node `ExistingParent`"
        );
    }

    #[test]
    fn test_check_parent_is_node() {
        let node_dec = Rc::new(RefCell::new(Dec::Node {
            name: "SomeNode".into(),
            parents: vec![],
            decs: vec![],
        }));
        let mut decs = LinkedHashMap::new();

        let result = TypeChecker::new().check_parent_is_node(&node_dec, &mut decs);
        match result {
            Err(_) => panic!("expected Ok"),
            Ok(is_node) => assert!(is_node),
        }
    }

    #[test]
    fn test_check_parent_is_not_node() {
        let enum_dec = Rc::new(RefCell::new(Dec::Enum {
            name: "SomeEnum".into(),
            values: vec![],
        }));
        let mut decs = LinkedHashMap::new();

        let result = TypeChecker::new().check_parent_is_node(&enum_dec, &mut decs);
        match result {
            Err(_) => panic!("expected Ok"),
            Ok(is_node) => assert!(!is_node),
        }
    }

    #[test]
    fn test_check_circular_inheritance() {
        let node_dec = Rc::new(RefCell::new(Dec::Node {
            name: "A".into(),
            parents: vec![],
            decs: vec![],
        }));
        let mut decs = LinkedHashMap::new();
        decs.insert(Rc::as_ptr(&node_dec), Rc::clone(&node_dec));

        let node_name = "A".to_string();
        let result = TypeChecker::new().check_no_circular_inheritance(&node_dec, &node_name, &decs);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "circular inheritance: `A` is its own parent: `A -> A`"
        );
    }
}
