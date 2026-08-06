use std::fs;
use std::path::PathBuf;

use minijinja::{context, Environment, Template, Value};

use crate::config::ast::Config;

fn input_name(input: &PathBuf) -> String {
    match input.file_name() {
        Some(s) => s.to_string_lossy().into(),
        None => String::from("<input>"),
    }
}

fn prepare_env(env: &mut Environment) {
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);
}

fn load_template<'a>(env: &'a mut Environment, template: &PathBuf) -> Template<'a, 'a> {
    let source = fs::read_to_string(template).expect("cannot read file");
    let name = input_name(template);
    env.add_template_owned(name.clone(), source)
        .expect("unable to load template");
    env.get_template(&name).unwrap()
}

fn render_template(template: &Template, ctx: &Value) -> String {
    template.render(ctx).expect("unable to render template")
}

fn build_context(config: &Config) -> Value {
    context! {
        config => config,
    }
}

fn remove_template_extension(basename: &str) -> &str {
    let exts = [".jinja", ".jinja2", ".j2"];
    for ext in exts {
        if !basename.ends_with(ext) {
            continue;
        }
        let len = basename.len() - ext.len();
        return &basename[0..len];
    }
    basename
}

pub fn render(config: &Config, template: &Option<PathBuf>) {
    let mut env = Environment::new();
    prepare_env(&mut env);
    let mut to_render = Vec::new();
    match template {
        Some(t) => {
            let tmpl = load_template(&mut env, t);
            let name = remove_template_extension(tmpl.name());
            let ctx = build_context(config);
            to_render.push((String::from(name), tmpl, ctx))
        }
        None => {}
    }
    to_render
        .iter()
        .map(|(n, t, c)| (n, render_template(t, c)))
        .for_each(|(n, r)| println!("{}:\n{}", n, r));
}
