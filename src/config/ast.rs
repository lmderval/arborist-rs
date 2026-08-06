use std::collections::HashMap;
use std::path::PathBuf;

pub enum Ty {
    Name(String),
    Int(u8),
    Str,
}

pub struct Type<'a> {
    pub dec: Option<&'a Dec<'a>>,
    pub ty: Ty,
}

impl<'a> Type<'a> {
    pub fn new(ty: Ty) -> Self {
        Self { dec: None, ty: ty }
    }

    pub fn define(&'a mut self, dec: &'a Dec<'a>) {
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

pub enum Dec<'a> {
    Node {
        name: String,
        parents: Vec<Type<'a>>,
        decs: Vec<Dec<'a>>,
    },
    Enum {
        name: String,
        values: Vec<String>,
    },
    Attr {
        name: String,
        modifier: AttrMod,
        ty: Type<'a>,
    },
}

pub enum Value {
    Int(i64),
    Str(String),
}

pub struct Exp<'a> {
    pub ty: Option<&'a Type<'a>>,
    pub value: Value,
}

impl<'a> Exp<'a> {
    pub fn new(value: Value) -> Self {
        Self {
            ty: None,
            value: value,
        }
    }
}

pub struct Var<'a> {
    pub name: String,
    pub exp: Exp<'a>,
}

impl<'a> Var<'a> {
    pub fn new(name: String, exp: Exp<'a>) -> Self {
        Self {
            name: name,
            exp: exp,
        }
    }
}

pub struct Config<'a> {
    pub path: Option<&'a PathBuf>,
    pub decs: Vec<Dec<'a>>,
    pub vars: HashMap<String, Var<'a>>,
}

impl<'a> Config<'a> {
    pub fn new(decs: Vec<Dec<'a>>, vars: HashMap<String, Var<'a>>) -> Self {
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
