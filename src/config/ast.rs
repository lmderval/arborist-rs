pub enum Ty {
    Name(String),
    Int(u8),
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

pub struct Config<'a> {
    pub decs: Vec<Dec<'a>>,
}

impl<'a> Config<'a> {
    pub fn new(decs: Vec<Dec<'a>>) -> Self {
        Self { decs: decs }
    }
}
