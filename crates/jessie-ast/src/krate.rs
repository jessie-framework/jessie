use crate::{
    ParseError,
    parser::{ItemKind, LowerItemKind},
};
use std::path::{Path, PathBuf};

use crate::parser::{Document, Parser};
use jessie_ast_lowering::LowerAst;
use jessie_cratesio::Dependency;
use jessie_span::SourceMap;
use jessie_tokenizer::Tokenizer;

#[derive(Debug, Clone)]
pub struct Crate<K = ItemKind> {
    pub name: String,
    pub module: Module<K>,
    pub deps: Vec<Crate<K>>,
    pub is_binary: bool,
}
impl<'a> LowerAst<'a> for Crate {
    type LowerTy = Crate<LowerItemKind>;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        Crate {
            name: self.name,
            module: self.module.lower(ctx),
            deps: self.deps.into_iter().map(|v| v.lower(ctx)).collect(),
            is_binary: self.is_binary,
        }
    }
    
}

impl<'a> LowerAst<'a> for Crate<LowerItemKind> {
    type LowerTy = CrateImportsResolved;
    fn lower(self, ctx : &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        todo!()
    }
}

pub struct CrateImportsResolved;

#[derive(Debug, Clone)]
pub struct Module<K = ItemKind> {
    pub name: String,
    pub children: Vec<Module<K>>,
    pub document: Option<Result<Document<K>>>,
}

impl<'a> LowerAst<'a> for Module {
    type LowerTy = Module<LowerItemKind>;
    fn lower(self, ctx: &'a mut jessie_ast_lowering::AstLowerCtx) -> Self::LowerTy {
        Module {
            name: self.name,
            children: self.children.into_iter().map(|v| v.lower(ctx)).collect(),
            document: self.document.map(|r| r.map(|m| m.lower(ctx))),
        }
    }
}

pub fn parse_crates(input: Dependency, sm: &mut SourceMap) -> Result<Crate> {
    let Dependency {
        version: _,
        name,
        path: _,
        entry,
        is_binary,
        dependencies,
    } = input;

    let module = parse_module(entry, sm)?;

    let mut deps = vec![];

    for dep in dependencies {
        deps.push(parse_crates(dep.clone(), sm)?);
    }

    Ok(Crate {
        name,
        module,
        deps,
        is_binary,
    })
}

fn parse_module(path: PathBuf, sm: &mut SourceMap) -> Result<Module> {
    let mut children = vec![];
    let name = path
        .file_prefix()
        .ok_or(ParseError::NoFilePrefix)?
        .to_string_lossy()
        .to_string();
    let document = parse_document(&path, sm);
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
                    children.push(parse_module(path.join(module_path.value()), sm)?);
                    is_custom_path = true;
                    break;
                }
            }
            if !is_custom_path && let Some(path) = path.parent() {
                let module_name = module.ident.to_string();
                if std::fs::exists(path.join(format!("{module_name}.rs")))
                    .map_err(|_| ParseError::CouldntOpenFile)?
                {
                    children.push(parse_module(path.join(format!("{module_name}.rs")), sm)?);
                } else {
                    children.push(parse_module(path.join(module_name).join("mod.rs"), sm)?);
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
    let contents = std::fs::read_to_string(path).map_err(|_| ParseError::CouldntOpenFile)?;
    syn::parse_file(&contents).map_err(|_| ParseError::CouldntParse)
}

fn parse_document(path: &Path, sm: &mut SourceMap) -> Option<Result<Document>> {
    let path = path.with_extension("jessie");
    let Ok((src, sp)) = sm.open(&path) else {
        return None;
    };
    let mut tok = Tokenizer::new(src, sp);
    let mut parser = Parser::new(tok.collect(), sm);
    Some(parser.parse_doc())
}
