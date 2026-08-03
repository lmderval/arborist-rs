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
    fn visit_node_dec(&mut self, name: &String, parents: &Vec<Type>) {
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
        self.output.push_str(" {}\n\n");
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

    fn visit_name_ty(&mut self, name: &String) {
        self.output.push_str(name);
    }
}
