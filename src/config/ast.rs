use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

pub enum Ty {
    Name(String),
    Int(u8),
    Str,
    Loc,
    Array(Rc<RefCell<Type>>),
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Name(name) => write!(f, "{}", name),
            Ty::Int(size) => write!(f, "i{}", size),
            Ty::Str => write!(f, "str"),
            Ty::Loc => write!(f, "loc"),
            Ty::Array(ty) => write!(f, "arr<{}>", ty.borrow().ty),
        }
    }
}

pub struct Type {
    pub dec: Option<Weak<RefCell<Dec>>>,
    pub ty: Ty,
}

impl Type {
    pub fn new(ty: Ty) -> Self {
        Self { dec: None, ty: ty }
    }

    pub fn define(&mut self, dec: Weak<RefCell<Dec>>) {
        self.dec = Some(dec);
    }
}

pub enum AttrMod {
    None,
    Readable,
    Writable,
    Mutable,
    Access(bool, bool, bool),
}

pub enum Dec {
    Node {
        name: String,
        parents: Vec<Rc<RefCell<Type>>>,
        decs: Vec<Rc<RefCell<Dec>>>,
    },
    Enum {
        name: String,
        values: Vec<String>,
    },
    Attr {
        name: String,
        modifier: AttrMod,
        ty: Rc<RefCell<Type>>,
    },
}

impl Dec {
    pub fn name(&self) -> &str {
        match self {
            Dec::Node { name, .. } => name,
            Dec::Enum { name, .. } => name,
            Dec::Attr { name, .. } => name,
        }
    }
}

pub enum Value {
    Int(i64),
    Str(String),
    Array(Vec<Exp>),
}

pub struct Exp {
    pub ty: Option<Rc<RefCell<Type>>>,
    pub value: Value,
}

impl Exp {
    pub fn new(value: Value) -> Self {
        Self {
            ty: None,
            value: value,
        }
    }

    pub fn type_exp(&mut self, ty: Rc<RefCell<Type>>) {
        self.ty = Some(ty);
    }
}

pub struct Var {
    pub name: String,
    pub exp: Exp,
}

impl Var {
    pub fn new(name: String, exp: Exp) -> Self {
        Self {
            name: name,
            exp: exp,
        }
    }
}

pub struct Config<'a> {
    pub path: Option<&'a PathBuf>,
    pub decs: Vec<Rc<RefCell<Dec>>>,
    pub vars: HashMap<String, Var>,
}

impl<'a> Config<'a> {
    pub fn new(decs: Vec<Rc<RefCell<Dec>>>, vars: HashMap<String, Var>) -> Self {
        Self {
            path: None,
            decs: decs,
            vars: vars,
        }
    }

    pub fn set_path(&mut self, path: &'a PathBuf) {
        self.path = Some(path)
    }
}
