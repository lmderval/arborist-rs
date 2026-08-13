use serde_core::ser::SerializeStruct;
use serde_core::Serialize;

use crate::config::ast::*;

macro_rules! serialize_kind {
    ( $state:expr; $kind:expr ) => {
        $state.serialize_field("kind", $kind)?
    };
}

impl Serialize for Ty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        match self {
            Self::Name(n) => {
                let mut state = serializer.serialize_struct("NameTy", 2)?;
                serialize_kind!(state; "Name");
                state.serialize_field("name", n)?;
                state.end()
            }
            Self::Int(s) => {
                let mut state = serializer.serialize_struct("IntTy", 2)?;
                serialize_kind!(state; "Int");
                state.serialize_field("size", s)?;
                state.end()
            }
            Self::Str => {
                let mut state = serializer.serialize_struct("StrTy", 1)?;
                serialize_kind!(state; "Str");
                state.end()
            }
            Self::Loc => {
                let mut state = serializer.serialize_struct("LocTy", 1)?;
                serialize_kind!(state; "Loc");
                state.end()
            }
            Self::Array(t) => {
                let mut state = serializer.serialize_struct("Array", 2)?;
                serialize_kind!(state; "Array");
                state.serialize_field("type", t)?;
                state.end()
            }
        }
    }
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Type", 2)?;
        state.serialize_field("ty", &self.ty)?;
        state.serialize_field("dec", &self.dec)?;
        state.end()
    }
}

macro_rules! serialize_access {
    ( $state:expr; [$r:expr, $w:expr, $m:expr] ) => {{
        $state.serialize_field("r", &$r)?;
        $state.serialize_field("w", &$w)?;
        $state.serialize_field("m", &$m)?;
    }};
}

impl Serialize for AttrMod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("AttrMod", 3)?;
        match self {
            Self::None => serialize_access!(state; [false, false, false]),
            Self::Readable => serialize_access!(state; [true, false, false]),
            Self::Writable => serialize_access!(state; [true, true, false]),
            Self::Mutable => serialize_access!(state; [true, true, true]),
            Self::Access(r, w, m) => serialize_access!(state; [r, w, m]),
        };
        state.end()
    }
}

impl Serialize for Dec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        match self {
            Self::Node {
                name,
                parents,
                decs,
            } => {
                let mut state = serializer.serialize_struct("NodeDec", 4)?;
                serialize_kind!(state; "Node");
                state.serialize_field("name", name)?;
                state.serialize_field("parents", parents)?;
                state.serialize_field("decs", decs)?;
                state.end()
            }
            Self::Enum { name, values } => {
                let mut state = serializer.serialize_struct("EnumDec", 3)?;
                serialize_kind!(state; "Enum");
                state.serialize_field("name", name)?;
                state.serialize_field("values", values)?;
                state.end()
            }
            Self::Attr { name, modifier, ty } => {
                let mut state = serializer.serialize_struct("AttrDec", 4)?;
                serialize_kind!(state; "Attr");
                state.serialize_field("name", name)?;
                state.serialize_field("modifier", modifier)?;
                state.serialize_field("ty", ty)?;
                state.end()
            }
        }
    }
}

macro_rules! serialize_value {
    ( $state:expr; $value:expr ) => {
        $state.serialize_field("value", $value)?
    };
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        match self {
            Self::Int(n) => {
                let mut state = serializer.serialize_struct("IntValue", 2)?;
                serialize_kind!(state; "Int");
                serialize_value!(state; n);
                state.end()
            }
            Self::Str(s) => {
                let mut state = serializer.serialize_struct("StrValue", 2)?;
                serialize_kind!(state; "Str");
                serialize_value!(state; s);
                state.end()
            }
            Self::Array(v) => {
                let mut state = serializer.serialize_struct("Array", 2)?;
                serialize_kind!(state; "Array");
                serialize_value!(state; v);
                state.end()
            }
        }
    }
}

impl Serialize for Exp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Exp", 2)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("ty", &self.ty)?;
        state.end()
    }
}

impl Serialize for Var {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Var", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("exp", &self.exp)?;
        state.end()
    }
}

impl Serialize for Config<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 2)?;
        state.serialize_field("decs", &self.decs)?;
        state.serialize_field("vars", &self.vars)?;
        state.end()
    }
}
