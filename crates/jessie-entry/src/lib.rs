use jessie_ast_lowering::{AstLowerCtx, LowerAst};
use jessie_span::SourceMap;

pub fn entry() -> String {
    let mut sm = SourceMap::new(); // We create a source map. This is a structure that contains the source text for every .jessie file.

    let dependencies = jessie_cratesio::entry();

    let krate =
        jessie_ast::krate::parse_crates(dependencies, &mut sm).expect("failed to parse crate");

    let mut lowerer = AstLowerCtx::new(&mut sm);

    let lowered_krate = krate.lower(&mut lowerer);

    panic!("{lowered_krate:#?}");
}
