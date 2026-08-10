use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

pub enum Ty {
    Name(String),
    Int(u8),
    Str,
    Array(Rc<Type>),
}

pub struct Type {
    pub dec: Option<Weak<Dec>>,
    pub ty: Ty,
}

impl Type {
    pub fn new(ty: Ty) -> Self {
        Self { dec: None, ty: ty }
    }

    pub fn define(&mut self, dec: Weak<Dec>) {
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
        parents: Vec<Rc<Type>>,
        decs: Vec<Rc<Dec>>,
    },
    Enum {
        name: String,
        values: Vec<String>,
    },
    Attr {
        name: String,
        modifier: AttrMod,
        ty: Rc<Type>,
    },
}

pub enum Value {
    Int(i64),
    Str(String),
    Array(Vec<Exp>),
}

pub struct Exp {
    pub ty: Option<Weak<Type>>,
    pub value: Value,
}

impl Exp {
    pub fn new(value: Value) -> Self {
        Self {
            ty: None,
            value: value,
        }
    }

    pub fn type_exp(&mut self, ty: Weak<Type>) {
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
    pub decs: Vec<Rc<Dec>>,
    pub vars: HashMap<String, Var>,
}

impl<'a> Config<'a> {
    pub fn new(decs: Vec<Rc<Dec>>, vars: HashMap<String, Var>) -> Self {
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
