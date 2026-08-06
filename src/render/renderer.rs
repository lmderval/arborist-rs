use std::path::PathBuf;
use std::{env, fs};

use minijinja::{context, path_loader, Environment, Error, Template, Value};

use crate::config::ast::{self, Config};

const TEMPLATE_EXTS: [&str; 3] = [".jinja", ".jinja2", ".j2"];

fn input_name(input: &PathBuf) -> String {
    match input.file_name() {
        Some(s) => s.to_string_lossy().into(),
        None => String::from("<input>"),
    }
}

fn multi_loader<F>(loaders: Vec<F>) -> impl Fn(&str) -> Result<Option<String>, Error>
where
    F: Fn(&str) -> Result<Option<String>, Error>,
{
    move |name| {
        let mut template_names = vec![String::from(name)];
        TEMPLATE_EXTS
            .iter()
            .for_each(|ext| template_names.push(format!("{}{}", name, ext)));
        for template_name in template_names {
            for loader in &loaders {
                match loader(&template_name) {
                    Ok(Some(result)) => {
                        return Ok(Some(result));
                    }
                    Ok(None) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
        }
        Ok(None)
    }
}

fn prepare_env(env: &mut Environment, config: &Config) {
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);
    let mut loaders = Vec::new();
    if let Ok(dir) = env::var("TEMPLATES_DIR") {
        loaders.push(path_loader(dir.into()));
    }
    if let Some(path) = &config.path {
        if let Some(ast::Var {
            exp:
                ast::Exp {
                    value: ast::Value::Str(subdir),
                    ..
                },
            ..
        }) = config.vars.get("templates_dir")
        {
            let dir = path.parent().unwrap().join(subdir);
            loaders.push(path_loader(dir));
        }
    }
    env.set_loader(multi_loader(loaders));
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
    for ext in TEMPLATE_EXTS {
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
    prepare_env(&mut env, config);
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
