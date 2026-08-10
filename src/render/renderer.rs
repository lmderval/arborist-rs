use std::path::PathBuf;
use std::{env, fs};

use minijinja::{context, path_loader, Environment, Error, Template, Value};

use crate::config::ast::{self, Config, Dec};
use crate::render::convert_case;

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

fn create_loaders(
    config: &Config,
) -> Vec<impl Fn(&str) -> Result<Option<String>, Error> + 'static> {
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
    loaders
}

fn prepare_env(env: &mut Environment, config: &Config) {
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);

    env.add_filter("snakecase", convert_case::to_snakecase);
    env.add_filter("kebabcase", convert_case::to_kebabcase);
    env.add_filter("uppercase", convert_case::to_uppercase);
    env.add_filter("pascalcase", convert_case::to_pascalcase);
    env.add_filter("camelcase", convert_case::to_camelcase);

    let loaders = create_loaders(config);
    env.set_loader(multi_loader(loaders));
}

fn load_template<'a>(env: &'a mut Environment, template: &PathBuf) -> Template<'a, 'a> {
    let name = input_name(template);
    if let Err(_) = env.get_template(&name) {
        let source = fs::read_to_string(template).expect("cannot read file");
        env.add_template_owned(name.clone(), source)
            .expect("unable to load template");
    }
    env.get_template(&name).unwrap()
}

fn find_node_templates<'a>(config: &'a Config<'a>) -> Vec<&'a str> {
    if let Some(ast::Var {
        exp:
            ast::Exp {
                value: ast::Value::Array(node_templates),
                ..
            },
        ..
    }) = config.vars.get("node_templates")
    {
        let templates = node_templates
            .iter()
            .filter_map(|node_template| match node_template {
                ast::Exp {
                    value: ast::Value::Str(template),
                    ..
                } => Some(template.as_str()),
                _ => None,
            })
            .collect();
        return templates;
    }
    Vec::new()
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

fn extract_extension(basename: &str) -> &str {
    match basename.find('.') {
        Some(i) => &basename[i..basename.len()],
        None => "",
    }
}

fn build_name(name: &str, ext: &str) -> String {
    format!("{}{}", convert_case::to_snakecase(name), ext)
}

fn build_context(filename: &str, config: &Config, node: Option<&Dec>) -> Value {
    context! {
        filename => filename,
        config => config,
        node => node,
    }
}

fn render_template(name: &str, template: &Template, ctx: &Value) {
    let rendered = template.render(ctx).expect("unable to render template");
    println!("{}:\n{}", name, rendered);
}

fn render_templates(env: &mut Environment, config: &Config) {
    if let Some(path) = &config.path {
        let parent = path.parent().unwrap();
        let templates = find_node_templates(config);
        for template in templates {
            for (name, node) in config.decs.iter().filter_map(|d| match &**d {
                Dec::Node { name, .. } => Some((name, d)),
                _ => None,
            }) {
                let tmpl = load_template(env, &parent.join(template));
                let ext = extract_extension(remove_template_extension(tmpl.name()));
                let filename = build_name(name, ext);
                let ctx = build_context(&filename, config, Some(node));
                render_template(&filename, &tmpl, &ctx);
            }
        }
    }
}

pub fn render(config: &Config, template: &Option<PathBuf>) {
    let mut env = Environment::new();
    prepare_env(&mut env, config);
    match template {
        Some(t) => {
            let tmpl = load_template(&mut env, t);
            let filename = remove_template_extension(tmpl.name());
            let ctx = build_context(&filename, config, None);
            render_template(filename, &tmpl, &ctx);
        }
        None => render_templates(&mut env, config),
    }
}
