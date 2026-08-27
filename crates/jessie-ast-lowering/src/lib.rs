use jessie_session::Session;

pub struct AstLowerCtx<'a> {
    pub sess: &'a mut Session,
}

impl<'a> AstLowerCtx<'a> {
    pub fn new(sess: &'a mut Session) -> Self {
        Self { sess }
    }
}

pub trait LowerAst<'a> {
    type LowerTy;
    fn lower(self, ctx: &'a mut AstLowerCtx) -> Self::LowerTy;
}
