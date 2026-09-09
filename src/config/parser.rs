use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive;

use crate::config::ast::*;
use crate::utils::error::MultiError;

#[derive(pest_derive::Parser)]
#[grammar = "arbolang.pest"]
pub struct ConfigParser;

impl ConfigParser {
    pub fn do_pass<'a>(source: String, _errors: &mut MultiError, file: &'a PathBuf) -> Config<'a> {
        let mut config = Self::parse_config(&source).expect("unsuccessful parse");
        config.set_path(file);
        config
    }

    fn parse_config<'a>(source: &str) -> Result<Config<'a>, Error<Rule>> {
        let mut decs = Vec::new();
        let mut vars = HashMap::new();
        let pairs = ConfigParser::parse(Rule::Config, source)?;
        for pair in pairs {
            match pair.as_rule() {
                Rule::Dec => decs.push(Rc::new(RefCell::new(Self::parse_dec(pair)))),
                Rule::Var => {
                    let var = Self::parse_var(pair);
                    vars.insert(var.name.clone(), var);
                }
                Rule::EOI => {}
                unknown => panic!("unknown config: {:?}", unknown),
            }
        }
        Ok(Config::new(decs, vars))
    }

    fn parse_var(pair: Pair<Rule>) -> Var {
        let mut pairs = pair.into_inner();
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let exp_pair = pairs.next().unwrap();
        let exp = Self::parse_exp(exp_pair);
        Var::new(name, exp)
    }

    fn parse_exp(pair: Pair<Rule>) -> Exp {
        match pair.as_rule() {
            Rule::Exp => Self::parse_exp(pair.into_inner().next().unwrap()),
            Rule::IntExp => Self::parse_int_exp(pair),
            Rule::StringExp => Self::parse_string_exp(pair),
            Rule::ArrayExp => Self::parse_array_exp(pair),
            unknown => panic!("unknown exp: {:?}", unknown),
        }
    }

    fn parse_int_exp(pair: Pair<Rule>) -> Exp {
        let value = pair.as_str().parse().expect("unable to parse integer");
        Exp::new(Value::Int(value))
    }

    fn parse_string_exp(pair: Pair<Rule>) -> Exp {
        let content_pair = pair.into_inner().next().unwrap();
        let content = String::from(content_pair.as_str());
        Exp::new(Value::Str(content))
    }

    fn parse_array_exp(pair: Pair<Rule>) -> Exp {
        let vec = pair.into_inner().map(|p| Self::parse_exp(p)).collect();
        Exp::new(Value::Array(vec))
    }

    fn parse_dec(pair: Pair<Rule>) -> Dec {
        match pair.as_rule() {
            Rule::Dec => Self::parse_dec(pair.into_inner().next().unwrap()),
            Rule::NodeDec => Self::parse_node_dec(pair),
            Rule::EnumDec => Self::parse_enum_dec(pair),
            Rule::AttrDec => Self::parse_attr_dec(pair),
            unknown => panic!("unknown dec: {:?}", unknown),
        }
    }

    fn parse_node_dec(pair: Pair<Rule>) -> Dec {
        let mut pairs = pair.into_inner();
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let parents_pair = pairs.next().unwrap();
        let parents = parents_pair
            .into_inner()
            .map(|p| Box::new(Type::new(Self::parse_ty(p))))
            .collect();
        let decs_pair = pairs.next().unwrap();
        let decs = decs_pair
            .into_inner()
            .map(|p| Rc::new(RefCell::new(Self::parse_dec(p))))
            .collect();
        Dec::Node {
            name,
            parents,
            decs,
        }
    }

    fn parse_enum_dec(pair: Pair<Rule>) -> Dec {
        let mut pairs = pair.into_inner();
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let values_pair = pairs.next().unwrap();
        let values = values_pair
            .into_inner()
            .map(|p| String::from(p.as_str()))
            .collect();
        Dec::Enum { name, values }
    }

    fn parse_attr_dec(pair: Pair<Rule>) -> Dec {
        let mut pairs = pair.into_inner();
        let modifier_pair = pairs.next().unwrap();
        let modifier = Self::parse_modifier(modifier_pair);
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let ty_pair = pairs.next().unwrap();
        let ty = Box::new(Type::new(Self::parse_ty(ty_pair)));
        Dec::Attr { name, modifier, ty }
    }

    fn parse_modifier(pair: Pair<Rule>) -> AttrMod {
        match pair.as_rule() {
            Rule::AttrMod => Self::parse_modifier(pair.into_inner().next().unwrap()),
            Rule::UserAttrMod => Self::parse_user_modifier(pair),
            Rule::PrimitiveAttrMod => Self::parse_primitive_modifier(pair),
            unknown => panic!("unknown modifier: {:?}", unknown),
        }
    }

    fn parse_user_modifier(pair: Pair<Rule>) -> AttrMod {
        let access_pair = pair.into_inner().next().unwrap();
        access_pair.as_str().chars().fold(
            AttrMod::Access(false, false, false),
            |res, c| match res {
                AttrMod::Access(r, w, m) => match c {
                    'r' => AttrMod::Access(true, w, m),
                    'w' => AttrMod::Access(r, true, m),
                    'm' => AttrMod::Access(r, w, true),
                    unknown => panic!("unknown user modifier: {:?}", unknown),
                },
                _ => unreachable!(),
            },
        )
    }

    fn parse_primitive_modifier(pair: Pair<Rule>) -> AttrMod {
        match pair.as_str() {
            "" => AttrMod::None,
            "readable" => AttrMod::Readable,
            "writable" => AttrMod::Writable,
            "mutable" => AttrMod::Mutable,
            unknown => panic!("unknown primitive modifier: {:?}", unknown),
        }
    }

    fn parse_ty(pair: Pair<Rule>) -> Ty {
        match pair.as_rule() {
            Rule::Type => Self::parse_ty(pair.into_inner().next().unwrap()),
            Rule::TypeId => Ty::Name(String::from(pair.as_str())),
            Rule::PrimitiveType => Self::parse_primitive_ty(pair),
            Rule::CollectionType => Self::parse_collection_ty(pair),
            unknown => panic!("unknown type: {:?}", unknown),
        }
    }

    fn parse_primitive_ty(pair: Pair<Rule>) -> Ty {
        match pair.as_str() {
            "i8" => Ty::Int(8),
            "i16" => Ty::Int(16),
            "i32" => Ty::Int(32),
            "i64" => Ty::Int(64),
            "str" => Ty::Str,
            "loc" => Ty::Loc,
            unknown => panic!("unknown primitive type: {:?}", unknown),
        }
    }

    fn parse_collection_ty(pair: Pair<Rule>) -> Ty {
        let mut pairs = pair.into_inner();
        let id = pairs.next().unwrap().as_str();
        let ty_pair = pairs.next().unwrap();
        let ty = Box::new(Type::new(Self::parse_ty(ty_pair)));
        match id {
            "arr" => Ty::Array(ty),
            unknown => panic!("unknown collection type: {:?}", unknown),
        }
    }
}
