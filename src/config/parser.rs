use pest::{self, Parser};
use pest_derive;

use crate::config::ast::*;

#[derive(pest_derive::Parser)]
#[grammar = "arbolang.pest"]
struct ConfigParser;

pub fn parse(source: &str) -> std::result::Result<Config<'_>, pest::error::Error<Rule>> {
    let mut decs = Vec::new();
    let pairs = ConfigParser::parse(Rule::Config, source)?;
    for pair in pairs {
        if let Rule::Dec = pair.as_rule() {
            decs.push(parse_dec(pair));
        }
    }
    Ok(Config::new(decs))
}

fn parse_dec(pair: pest::iterators::Pair<Rule>) -> Dec {
    match pair.as_rule() {
        Rule::Dec => parse_dec(pair.into_inner().next().unwrap()),
        Rule::NodeDec => parse_node_dec(pair),
        Rule::EnumDec => parse_enum_dec(pair),
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
        .map(|p| Type::new(Ty::Name(String::from(p.as_str()))))
        .collect();
    Dec::Node {
        name: name,
        parents: parents,
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
