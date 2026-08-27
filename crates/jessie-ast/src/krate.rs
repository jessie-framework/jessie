use crate::{ParseError, parser::ItemKind};
use std::path::{Path, PathBuf};

use crate::parser::{Document, Parser};
use jessie_cratesio::Dependency;
use jessie_tokenizer::Tokenizer;

#[derive(Debug, Clone)]
pub struct Crate<K = ItemKind> {
    pub name: String,
    pub module: Module<K>,
    pub deps: Vec<Crate<K>>,
    pub is_binary: bool,
}

#[derive(Debug, Clone)]
pub struct Module<K = ItemKind> {
    pub name: String,
    pub children: Vec<Module<K>>,
    pub document: Option<Document<K>>,
}

pub fn parse_crates(input: Dependency, sess: &mut jessie_session::Session) -> Result<Crate> {
    let Dependency {
        version: _,
        name,
        path: _,
        entry,
        is_binary,
        dependencies,
    } = input;

    let module = parse_module(entry, sess)?;

    let mut deps = vec![];

    for dep in dependencies {
        deps.push(parse_crates(dep.clone(), sess)?);
    }

    Ok(Crate {
        name,
        module,
        deps,
        is_binary,
    })
}

fn parse_module(path: PathBuf, sess: &mut jessie_session::Session) -> Result<Module> {
    let mut children = vec![];
    let name = path
        .file_prefix()
        .ok_or(ParseError::NoFilePrefix { path: path.clone() })?
        .to_string_lossy()
        .to_string();
    let document = parse_document(&path, sess);
    let file = parse_rs(path.clone())?;
    for item in file.items {
        if let syn::Item::Mod(module) = item
            && module.content.is_none()
        {
            let mut is_custom_path = false;
            for attr in module.attrs {
                if let syn::Meta::NameValue(name_value) = attr.meta
                    && let Some(ident) = name_value.path.get_ident()
                    && ident == "path"
                    && let syn::Expr::Lit(literal) = name_value.value
                    && let syn::Lit::Str(module_path) = literal.lit
                {
                    children.push(parse_module(path.join(module_path.value()), sess)?);
                    is_custom_path = true;
                    break;
                }
            }
            if !is_custom_path && let Some(path) = path.parent() {
                let module_name = module.ident.to_string();
                if std::fs::exists(path.join(format!("{module_name}.rs"))).map_err(|_| {
                    ParseError::CouldntOpenFile {
                        path: path.join(format!("{module_name}.rs")),
                    }
                })? {
                    children.push(parse_module(path.join(format!("{module_name}.rs")), sess)?);
                } else {
                    children.push(parse_module(path.join(module_name).join("mod.rs"), sess)?);
                }
            }
        }
    }
    Ok(Module {
        name,
        children,
        document,
    })
}

type Result<T> = std::result::Result<T, ParseError>;

fn parse_rs(path: PathBuf) -> Result<syn::File> {
    let contents = std::fs::read_to_string(path.clone())
        .map_err(|_| ParseError::CouldntOpenFile { path: path.clone() })?;
    syn::parse_file(&contents).map_err(|_| ParseError::CouldntParse { path })
}

fn parse_document(path: &Path, sess: &mut jessie_session::Session) -> Option<Document> {
    let path = path.with_extension("jessie");
    let Ok((src, sp)) = sess.sm.open(&path) else {
        return None;
    };
    let mut tok = Tokenizer::new(src, sp);
    let mut parser = Parser::new(tok.collect(), sess, sp);
    Some(parser.parse_doc())
}
