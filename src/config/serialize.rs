use serde_core::ser::SerializeStruct;
use serde_core::Serialize;

use crate::config::ast::*;

impl Serialize for Ty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        match self {
            Self::Name(n) => {
                let mut state = serializer.serialize_struct("NameTy", 2)?;
                state.serialize_field("kind", "Name")?;
                state.serialize_field("name", n)?;
                state.end()
            }
            Self::Int(s) => {
                let mut state = serializer.serialize_struct("IntTy", 2)?;
                state.serialize_field("kind", "Int")?;
                state.serialize_field("size", s)?;
                state.end()
            }
        }
    }
}

impl Serialize for Type<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Type", 1)?;
        state.serialize_field("ty", &self.ty)?;
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
            Self::Writable => serialize_access!(state; [false, true, false]),
            Self::Mutable => serialize_access!(state; [false, false, true]),
            Self::Access(r, w, m) => serialize_access!(state; [r, w, m]),
        };
        state.end()
    }
}

impl Serialize for Dec<'_> {
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
                state.serialize_field("kind", "Node")?;
                state.serialize_field("name", name)?;
                state.serialize_field("parents", parents)?;
                state.serialize_field("decs", decs)?;
                state.end()
            }
            Self::Enum { name, values } => {
                let mut state = serializer.serialize_struct("EnumDec", 3)?;
                state.serialize_field("kind", "Enum")?;
                state.serialize_field("name", name)?;
                state.serialize_field("values", values)?;
                state.end()
            }
            Self::Attr { name, modifier, ty } => {
                let mut state = serializer.serialize_struct("AttrDec", 4)?;
                state.serialize_field("kind", "Attr")?;
                state.serialize_field("name", name)?;
                state.serialize_field("modifier", modifier)?;
                state.serialize_field("ty", ty)?;
                state.end()
            }
        }
    }
}

impl Serialize for Config<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde_core::Serializer,
    {
        let mut state = serializer.serialize_struct("Config", 1)?;
        state.serialize_field("decs", &self.decs)?;
        state.end()
    }
}
