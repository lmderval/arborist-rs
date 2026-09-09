use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::{Parser, Span};
use pest_derive;

use crate::config::ast::*;
use crate::utils::error::{ErrorKind, MultiError};

#[derive(pest_derive::Parser)]
#[grammar = "arbolang.pest"]
pub struct ConfigParser<'err> {
    errors: &'err mut MultiError,
}

impl<'err> ConfigParser<'err> {
    fn new(errors: &'err mut MultiError) -> Self {
        Self { errors }
    }

    pub fn do_pass<'a>(source: String, errors: &mut MultiError, file: &'a PathBuf) -> Config<'a> {
        let mut config = match ConfigParser::new(errors).parse_config(&source) {
            Ok(config) => config,
            Err(err) => {
                errors.push(
                    ErrorKind::Syntax,
                    &format!("cannot parse configuration:\n{}", err),
                );
                Config::empty()
            }
        };
        config.set_path(file);
        config
    }

    fn error_at(&mut self, kind: ErrorKind, message: &str, reason: &str, span: Span) {
        let error = Error::<Rule>::new_from_span(
            ErrorVariant::CustomError {
                message: reason.into(),
            },
            span,
        );
        self.errors.push(kind, &format!("{}:\n{}", message, error));
    }

    fn parse_config<'a>(&mut self, source: &str) -> Result<Config<'a>, Error<Rule>> {
        let mut decs = Vec::new();
        let mut vars = HashMap::new();
        let pairs = Self::parse(Rule::Config, source)?;
        for pair in pairs {
            match pair.as_rule() {
                Rule::Dec => {
                    if let Some(dec) = self.parse_dec(pair) {
                        decs.push(Rc::new(RefCell::new(dec)))
                    }
                }
                Rule::Var => {
                    let var = self.parse_var(pair);
                    vars.insert(var.name.clone(), var);
                }
                Rule::EOI => {}
                _ => {
                    self.error_at(
                        ErrorKind::Internal,
                        "cannot parse configuration",
                        &format!("invalid top-level element `{}`", pair),
                        pair.as_span(),
                    );
                }
            }
        }
        Ok(Config::new(decs, vars))
    }

    fn parse_var(&mut self, pair: Pair<Rule>) -> Var {
        let mut pairs = pair.into_inner();
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let exp_pair = pairs.next().unwrap();
        let exp = self.parse_exp(exp_pair);
        Var::new(name, exp)
    }

    fn parse_exp(&mut self, pair: Pair<Rule>) -> Exp {
        match pair.as_rule() {
            Rule::Exp => self.parse_exp(pair.into_inner().next().unwrap()),
            Rule::IntExp => self.parse_int_exp(pair),
            Rule::StringExp => self.parse_string_exp(pair),
            Rule::ArrayExp => self.parse_array_exp(pair),
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse expression",
                    &format!("unknown expression: `{}`", pair),
                    pair.as_span(),
                );
                Exp::new(ERROR_VALUE)
            }
        }
    }

    fn parse_int_exp(&mut self, pair: Pair<Rule>) -> Exp {
        match pair.as_str().parse() {
            Ok(value) => Exp::new(Value::Int(value)),
            Err(err) => {
                self.error_at(
                    ErrorKind::Syntax,
                    "cannot parse integer",
                    &err.to_string(),
                    pair.as_span(),
                );
                Exp::new(ERROR_VALUE)
            }
        }
    }

    fn parse_string_exp(&mut self, pair: Pair<Rule>) -> Exp {
        let content_pair = pair.into_inner().next().unwrap();
        let content = String::from(content_pair.as_str());
        Exp::new(Value::Str(content))
    }

    fn parse_array_exp(&mut self, pair: Pair<Rule>) -> Exp {
        let vec = pair.into_inner().map(|p| self.parse_exp(p)).collect();
        Exp::new(Value::Array(vec))
    }

    fn parse_dec(&mut self, pair: Pair<Rule>) -> Option<Dec> {
        match pair.as_rule() {
            Rule::Dec => self.parse_dec(pair.into_inner().next().unwrap()),
            Rule::NodeDec => Some(self.parse_node_dec(pair)),
            Rule::EnumDec => Some(self.parse_enum_dec(pair)),
            Rule::AttrDec => Some(self.parse_attr_dec(pair)),
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse declaration",
                    &format!("invalid declaration `{}`", pair),
                    pair.as_span(),
                );
                None
            }
        }
    }

    fn parse_node_dec(&mut self, pair: Pair<Rule>) -> Dec {
        let mut pairs = pair.into_inner();
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let parents_pair = pairs.next().unwrap();
        let parents = parents_pair
            .into_inner()
            .map(|p| Box::new(Type::new(self.parse_ty(p))))
            .collect();
        let decs_pair = pairs.next().unwrap();
        let decs = decs_pair
            .into_inner()
            .filter_map(|p| self.parse_dec(p))
            .map(|d| Rc::new(RefCell::new(d)))
            .collect();
        Dec::Node {
            name,
            parents,
            decs,
        }
    }

    fn parse_enum_dec(&mut self, pair: Pair<Rule>) -> Dec {
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

    fn parse_attr_dec(&mut self, pair: Pair<Rule>) -> Dec {
        let mut pairs = pair.into_inner();
        let modifier_pair = pairs.next().unwrap();
        let modifier = self.parse_modifier(modifier_pair);
        let name_pair = pairs.next().unwrap();
        let name = String::from(name_pair.as_str());
        let ty_pair = pairs.next().unwrap();
        let ty = Box::new(Type::new(self.parse_ty(ty_pair)));
        Dec::Attr { name, modifier, ty }
    }

    fn parse_modifier(&mut self, pair: Pair<Rule>) -> AttrMod {
        match pair.as_rule() {
            Rule::AttrMod => self.parse_modifier(pair.into_inner().next().unwrap()),
            Rule::UserAttrMod => self.parse_user_modifier(pair),
            Rule::PrimitiveAttrMod => self.parse_primitive_modifier(pair),
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse modifier",
                    &format!("invalid modifier `{}`", pair),
                    pair.as_span(),
                );
                AttrMod::None
            }
        }
    }

    fn parse_user_modifier(&mut self, pair: Pair<Rule>) -> AttrMod {
        let access_pair = pair.into_inner().next().unwrap();
        access_pair.as_str().chars().fold(
            AttrMod::Access(false, false, false),
            |res, c| match res {
                AttrMod::Access(r, w, m) => match c {
                    'r' => AttrMod::Access(true, w, m),
                    'w' => AttrMod::Access(r, true, m),
                    'm' => AttrMod::Access(r, w, true),
                    _ => {
                        self.error_at(
                            ErrorKind::Internal,
                            "cannot parse user modifier",
                            &format!("invalid user modifier flag: `{}`", c),
                            access_pair.as_span(),
                        );
                        res
                    }
                },
                _ => unreachable!(),
            },
        )
    }

    fn parse_primitive_modifier(&mut self, pair: Pair<Rule>) -> AttrMod {
        match pair.as_str() {
            "" => AttrMod::None,
            "readable" => AttrMod::Readable,
            "writable" => AttrMod::Writable,
            "mutable" => AttrMod::Mutable,
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse primitive modifier",
                    &format!("invalid primitive modifier `{}`", pair),
                    pair.as_span(),
                );
                AttrMod::None
            }
        }
    }

    fn parse_ty(&mut self, pair: Pair<Rule>) -> Ty {
        match pair.as_rule() {
            Rule::Type => self.parse_ty(pair.into_inner().next().unwrap()),
            Rule::TypeId => Ty::Name(String::from(pair.as_str())),
            Rule::PrimitiveType => self.parse_primitive_ty(pair),
            Rule::CollectionType => self.parse_collection_ty(pair),
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse type",
                    &format!("invalid type `{}`", pair),
                    pair.as_span(),
                );
                ERROR_TY
            }
        }
    }

    fn parse_primitive_ty(&mut self, pair: Pair<Rule>) -> Ty {
        match pair.as_str() {
            "i8" => Ty::Int(8),
            "i16" => Ty::Int(16),
            "i32" => Ty::Int(32),
            "i64" => Ty::Int(64),
            "str" => Ty::Str,
            "loc" => Ty::Loc,
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse primitive type",
                    &format!("invalid primitive type `{}`", pair),
                    pair.as_span(),
                );
                ERROR_TY
            }
        }
    }

    fn parse_collection_ty(&mut self, pair: Pair<Rule>) -> Ty {
        let mut pairs = pair.into_inner();
        let id_pair = pairs.next().unwrap();
        let ty_pair = pairs.next().unwrap();
        let ty = Box::new(Type::new(self.parse_ty(ty_pair)));
        match id_pair.as_str() {
            "arr" => Ty::Array(ty),
            _ => {
                self.error_at(
                    ErrorKind::Internal,
                    "cannot parse collection type",
                    &format!("invalid collection type `{}`", id_pair),
                    id_pair.as_span(),
                );
                ERROR_TY
            }
        }
    }
}
