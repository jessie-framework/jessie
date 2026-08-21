use jessie_span::SourceMap;

pub struct AstLowerCtx<'a> {
    pub sm: &'a mut SourceMap,
    idx: u32,
}

impl<'a> AstLowerCtx<'a> {
    pub fn new(sm: &'a mut SourceMap) -> Self {
        let idx = 0;
        Self { sm, idx }
    }

    pub fn new_did(&mut self) -> DefId {
        self.idx += 1;
        DefId(self.idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(u32);

impl DefId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

pub trait LowerAst<'a> {
    type LowerTy;
    fn lower(self, ctx: &'a mut AstLowerCtx) -> Self::LowerTy;
}
