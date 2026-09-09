use std::cell::RefCell;
use std::rc::Rc;

use crate::config::ast::*;
use crate::config::visitor::VoidVisitor;
use crate::utils::error::MultiError;

pub struct PrettyPrinter {
    output: String,
    indent: usize,
}

impl PrettyPrinter {
    fn new() -> Self {
        PrettyPrinter {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn do_pass<'a>(config: Config<'a>, _errors: &mut MultiError) -> Config<'a> {
        Self::print(&config);
        config
    }

    pub fn print(config: &Config) {
        let mut printer = PrettyPrinter::new();
        printer.visit_config(config);
        println!("{}", printer.output.trim_end());
    }

    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }
}

impl VoidVisitor for PrettyPrinter {
    fn visit_var(&mut self, var: &Var) {
        self.output
            .push_str(&format!("{}{} = ", self.indent_str(), var.name));
        self.visit_exp(&var.exp);
        self.output.push_str(";\n\n");
    }

    fn visit_int_val(&mut self, val: &i64) {
        self.output.push_str(&format!("{}", val));
    }

    fn visit_string_val(&mut self, val: &String) {
        self.output.push_str(&format!("\"{}\"", val))
    }

    fn visit_array_val(&mut self, val: &Vec<Exp>) {
        match val {
            v if v.is_empty() => self.output.push_str("[]"),
            v if v.len() == 1 => {
                self.output.push_str("[ ");
                val.get(0).iter().for_each(|e| self.visit_exp(e));
                self.output.push_str(" ]");
            }
            _ => {
                self.output.push_str("[\n");
                self.indent += 1;
                val.iter().for_each(|e| {
                    self.output.push_str(&self.indent_str());
                    self.visit_exp(e);
                    self.output.push_str(",\n");
                });
                self.indent -= 1;
                self.output.push_str(&format!("{}]", self.indent_str()));
            }
        }
    }

    fn visit_node_dec(
        &mut self,
        name: &String,
        parents: &Vec<Box<Type>>,
        decs: &Vec<Rc<RefCell<Dec>>>,
    ) {
        self.output
            .push_str(&format!("{}node {}", self.indent_str(), name));
        match parents {
            v if v.is_empty() => {}
            v if v.len() == 1 => {
                self.output.push_str("(");
                self.visit_type(parents.get(0).unwrap());
                self.output.push_str(&format!("{})", self.indent_str()));
            }
            _ => {
                self.output.push_str("(\n");
                self.indent += 1;
                parents.iter().for_each(|p| {
                    self.output.push_str(&self.indent_str());
                    self.visit_type(p);
                    self.output.push_str(",\n");
                });
                self.indent -= 1;
                self.output.push_str(&format!("{})", self.indent_str()));
            }
        }
        if decs.is_empty() {
            self.output.push_str(" {}\n\n");
        } else {
            self.output.push_str(" {\n");
            self.indent += 1;
            decs.iter().for_each(|d| self.visit_dec(d));
            self.indent -= 1;
            self.output
                .push_str(&format!("{}}}\n\n", self.indent_str()));
        }
    }

    fn visit_enum_dec(&mut self, name: &String, values: &Vec<String>) {
        self.output.push_str(&format!("enum {} {{\n", name));
        self.indent += 1;
        values.iter().for_each(|v| {
            self.output
                .push_str(&format!("{}{},\n", self.indent_str(), v))
        });
        self.indent -= 1;
        self.output.push_str("}\n\n");
    }

    fn visit_attr_dec(&mut self, name: &String, modifier: &AttrMod, ty: &Box<Type>) {
        let modifier_str = match modifier {
            AttrMod::None => "",
            AttrMod::Readable => "readable ",
            AttrMod::Writable => "writable ",
            AttrMod::Mutable => "mutable ",
            AttrMod::Access(r, w, m) => {
                let mut access = String::new();
                if *r {
                    access.push('r');
                }
                if *w {
                    access.push('w');
                }
                if *m {
                    access.push('m');
                }
                &format!("access<{}> ", access)
            }
        };
        self.output.push_str(&format!(
            "{}{}{}: {};\n",
            self.indent_str(),
            modifier_str,
            name,
            ty.ty,
        ));
    }
}
