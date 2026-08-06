use crate::config::ast::*;
use crate::config::visitor::VoidVisitor;

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

    pub fn print_config(config: &Config) {
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

    fn visit_value(&mut self, value: &Value) {
        match value {
            Value::Int(n) => self.output.push_str(&format!("{}", n)),
            Value::Str(s) => self.output.push_str(&format!("\"{}\"", s)),
        }
    }

    fn visit_node_dec(&mut self, name: &String, parents: &Vec<Type>, decs: &Vec<Dec>) {
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

    fn visit_attr_dec(&mut self, name: &String, modifier: &AttrMod, ty: &Type) {
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
        self.output
            .push_str(&format!("{}{}{}: ", self.indent_str(), modifier_str, name));
        self.visit_type(ty);
        self.output.push_str(";\n");
    }

    fn visit_name_ty(&mut self, name: &String) {
        self.output.push_str(name);
    }

    fn visit_int_ty(&mut self, size: &u8) {
        self.output.push_str(&format!("i{}", size));
    }

    fn visit_str_ty(&mut self) {
        self.output.push_str("str");
    }
}
