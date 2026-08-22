use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use minijinja::value::from_args;
use minijinja::{value::Object, Error, State, Value};

#[derive(Debug)]
pub struct Joiner {
    first: Value,
    sep: Value,
    used: AtomicBool,
}

impl Joiner {
    fn new(first: Value, sep: Value) -> Self {
        Self {
            first,
            sep,
            used: AtomicBool::new(false),
        }
    }
}

impl Object for Joiner {
    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let () = from_args(args)?;
        if self.used.swap(true, Ordering::Relaxed) {
            Ok(self.sep.clone())
        } else {
            Ok(self.first.clone())
        }
    }
}

pub fn joiner(sep: Option<Value>, first: Option<Value>) -> Value {
    Value::from_object(Joiner::new(
        first.unwrap_or_else(|| Value::from("")),
        sep.unwrap_or_else(|| Value::from("")),
    ))
}

#[cfg(test)]
mod tests {
    use minijinja::{context, Environment};

    use super::*;

    macro_rules! test_case {
        ( $name:ident, $source:literal, $expected:literal ) => {
            #[test]
            fn $name() {
                let mut env = Environment::new();
                env.add_function("joiner", joiner);
                let Ok(tmpl) = env.template_from_str($source) else {
                    assert!(false);
                    return;
                };
                let Ok(content) = tmpl.render(context!()) else {
                    assert!(false);
                    return;
                };
                assert_eq!(content, $expected);
            }
        };
    }

    macro_rules! test_cases {
        { $( ( $name:ident; $source:literal ) => $expected:literal ),* } => {
            $(
                test_case!($name, $source, $expected);
            )*
        };
    }

    test_cases! {
        (default; r#"
{%- set sep = joiner() -%}
{{ sep() }}foo{{ sep() }}bar{{ sep() }}baz
"#
        ) => "foobarbaz",
        (separator; r#"
{%- set sep = joiner(", ") -%}
{{ sep() }}foo{{ sep() }}bar{{ sep() }}baz
"#
        ) => "foo, bar, baz",
        (separator_first; r#"
{%- set sep = joiner(", ", "> ") -%}
{{ sep() }}foo{{ sep() }}bar{{ sep() }}baz
"#
        ) => "> foo, bar, baz"
    }
}
