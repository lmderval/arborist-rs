use std::{cell::RefCell, collections::HashMap, rc::Rc};

use pest::{self, Parser};
use pest_derive;

use crate::config::ast::*;

#[derive(pest_derive::Parser)]
#[grammar = "arbolang.pest"]
struct ConfigParser;

pub fn parse(source: &str) -> std::result::Result<Config<'_>, pest::error::Error<Rule>> {
    let mut decs = Vec::new();
    let mut vars = HashMap::new();
    let pairs = ConfigParser::parse(Rule::Config, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::Dec => decs.push(Rc::new(RefCell::new(parse_dec(pair)))),
            Rule::Var => {
                let var = parse_var(pair);
                vars.insert(var.name.clone(), var);
            }
            Rule::EOI => {}
            unknown => panic!("unknown config: {:?}", unknown),
        }
    }
    Ok(Config::new(decs, vars))
}

fn parse_var(pair: pest::iterators::Pair<Rule>) -> Var {
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name = String::from(name_pair.as_str());
    let exp_pair = pairs.next().unwrap();
    let exp = parse_exp(exp_pair);
    Var::new(name, exp)
}

fn parse_exp(pair: pest::iterators::Pair<Rule>) -> Exp {
    match pair.as_rule() {
        Rule::Exp => parse_exp(pair.into_inner().next().unwrap()),
        Rule::IntExp => parse_int_exp(pair),
        Rule::StringExp => parse_string_exp(pair),
        Rule::ArrayExp => parse_array_exp(pair),
        unknown => panic!("unknown exp: {:?}", unknown),
    }
}

fn parse_int_exp(pair: pest::iterators::Pair<Rule>) -> Exp {
    let value = pair.as_str().parse().expect("unable to parse integer");
    Exp::new(Value::Int(value))
}

fn parse_string_exp(pair: pest::iterators::Pair<Rule>) -> Exp {
    let content_pair = pair.into_inner().next().unwrap();
    let content = String::from(content_pair.as_str());
    Exp::new(Value::Str(content))
}

fn parse_array_exp(pair: pest::iterators::Pair<Rule>) -> Exp {
    let vec = pair.into_inner().map(|p| parse_exp(p)).collect();
    Exp::new(Value::Array(vec))
}

fn parse_dec(pair: pest::iterators::Pair<Rule>) -> Dec {
    match pair.as_rule() {
        Rule::Dec => parse_dec(pair.into_inner().next().unwrap()),
        Rule::NodeDec => parse_node_dec(pair),
        Rule::EnumDec => parse_enum_dec(pair),
        Rule::AttrDec => parse_attr_dec(pair),
        unknown => panic!("unknown dec: {:?}", unknown),
    }
}

fn parse_node_dec(pair: pest::iterators::Pair<Rule>) -> Dec {
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name = String::from(name_pair.as_str());
    let parents_pair = pairs.next().unwrap();
    let parents = parents_pair
        .into_inner()
        .map(|p| Rc::new(RefCell::new(Type::new(parse_ty(p)))))
        .collect();
    let decs_pair = pairs.next().unwrap();
    let decs = decs_pair
        .into_inner()
        .map(|p| Rc::new(RefCell::new(parse_dec(p))))
        .collect();
    Dec::Node {
        name: name,
        parents: parents,
        decs: decs,
    }
}

fn parse_enum_dec(pair: pest::iterators::Pair<Rule>) -> Dec {
    let mut pairs = pair.into_inner();
    let name_pair = pairs.next().unwrap();
    let name = String::from(name_pair.as_str());
    let values_pair = pairs.next().unwrap();
    let values = values_pair
        .into_inner()
        .map(|p| String::from(p.as_str()))
        .collect();
    Dec::Enum {
        name: name,
        values: values,
    }
}

fn parse_attr_dec(pair: pest::iterators::Pair<Rule>) -> Dec {
    let mut pairs = pair.into_inner();
    let modifier_pair = pairs.next().unwrap();
    let modifier = parse_modifier(modifier_pair);
    let name_pair = pairs.next().unwrap();
    let name = String::from(name_pair.as_str());
    let ty_pair = pairs.next().unwrap();
    let ty = Rc::new(RefCell::new(Type::new(parse_ty(ty_pair))));
    Dec::Attr {
        name: name,
        modifier: modifier,
        ty: ty,
    }
}

fn parse_modifier(pair: pest::iterators::Pair<Rule>) -> AttrMod {
    match pair.as_rule() {
        Rule::AttrMod => parse_modifier(pair.into_inner().next().unwrap()),
        Rule::UserAttrMod => parse_user_modifier(pair),
        Rule::PrimitiveAttrMod => parse_primitive_modifier(pair),
        unknown => panic!("unknown modifier: {:?}", unknown),
    }
}

fn parse_user_modifier(pair: pest::iterators::Pair<Rule>) -> AttrMod {
    let access_pair = pair.into_inner().next().unwrap();
    access_pair
        .as_str()
        .chars()
        .fold(AttrMod::Access(false, false, false), |res, c| match res {
            AttrMod::Access(r, w, m) => match c {
                'r' => AttrMod::Access(true, w, m),
                'w' => AttrMod::Access(r, true, m),
                'm' => AttrMod::Access(r, w, true),
                unknown => panic!("unknown user modifier: {:?}", unknown),
            },
            _ => unreachable!(),
        })
}

fn parse_primitive_modifier(pair: pest::iterators::Pair<Rule>) -> AttrMod {
    match pair.as_str() {
        "" => AttrMod::None,
        "readable" => AttrMod::Readable,
        "writable" => AttrMod::Writable,
        "mutable" => AttrMod::Mutable,
        unknown => panic!("unknown primitive modifier: {:?}", unknown),
    }
}

fn parse_ty(pair: pest::iterators::Pair<Rule>) -> Ty {
    match pair.as_rule() {
        Rule::Type => parse_ty(pair.into_inner().next().unwrap()),
        Rule::TypeId => Ty::Name(String::from(pair.as_str())),
        Rule::PrimitiveType => parse_primitive_ty(pair),
        Rule::CollectionType => parse_collection_ty(pair),
        unknown => panic!("unknown type: {:?}", unknown),
    }
}

fn parse_primitive_ty(pair: pest::iterators::Pair<Rule>) -> Ty {
    match pair.as_str() {
        "i8" => Ty::Int(8),
        "i16" => Ty::Int(16),
        "i32" => Ty::Int(32),
        "i64" => Ty::Int(64),
        "str" => Ty::Str,
        unknown => panic!("unknown primitive type: {:?}", unknown),
    }
}

fn parse_collection_ty(pair: pest::iterators::Pair<Rule>) -> Ty {
    let mut pairs = pair.into_inner();
    let id = pairs.next().unwrap().as_str();
    let ty_pair = pairs.next().unwrap();
    let ty = Rc::new(RefCell::new(Type::new(parse_ty(ty_pair))));
    match id {
        "arr" => Ty::Array(ty),
        unknown => panic!("unknown collection type: {:?}", unknown),
    }
}
