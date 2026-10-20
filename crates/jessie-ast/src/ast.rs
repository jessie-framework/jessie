use jessie_span::{Span, Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    pub private_use_as_methods_instead: u32,
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub enum TyKind {
    Impl(Vec<Bound>),
    TraitObject(Vec<Bound>, TraitObjectSyntax),
    TyPath(TyPath),
    Tuple(Vec<Ty>),
    Never,
    RawPtr(MutTy),
    Ref(Option<Lifetime>, MutTy),
    Array(Box<Ty>, Box<Expr>),
    Slice(Box<Ty>),
    Inferred,
    QPath(QSelf, TyPath),
    FnPtr(FnPtrTy),
    Err,
}

#[derive(Debug, Clone)]
pub struct FnPtrTy {
    pub modifier: Option<TraitBoundModifier>,
    pub unsafeness: Unsafeness,
    pub ext: Option<Extern>,
    pub params: Vec<Ty>,
    pub ret_ty: Option<Box<Ty>>,
}

#[derive(Debug, Clone)]
pub struct Extern {
    pub abi: String,
}

#[derive(Debug, Clone)]
pub struct StrLit {
    pub span: Span,
    pub src: Symbol,
    pub suffix: Option<Symbol>,
}

#[derive(Debug, Clone)]
pub enum Unsafeness {
    Unsafe,
    Default,
}

#[derive(Debug, Clone)]
pub struct QSelf {
    pub ty: Box<Ty>,
    pub path: TyPath,
}

#[derive(Debug, Clone)]
pub struct MutTy {
    pub mutability: Mutability,
    pub ty: Box<Ty>,
}

#[derive(Debug, Clone)]
pub enum Mutability {
    Mut,
    Not,
}

#[derive(Debug, Clone)]
pub struct TyPath {
    pub is_root: bool,
    pub segments: Vec<TyPathSegment>,
}

#[derive(Debug, Clone)]
pub enum GenericArg {
    Lt(Lifetime),
    Ty(Ty),
    Const(Expr),
}

#[derive(Debug, Clone)]
pub enum Bound {
    Lifetime(Lifetime),
    TraitBound(TraitBound),
    UseBound(UseBound),
}

#[derive(Debug, Clone)]
pub struct UseBound {
    pub args: Vec<GenericArg>,
}

#[derive(Debug, Clone)]
pub struct TraitBound {
    pub modifier: Option<TraitBoundModifier>,
    pub path: TyPath,
}

#[derive(Debug, Clone)]
pub enum TraitBoundModifier {
    Qm,
    For(Vec<GenericArg>),
}

#[derive(Debug, Clone)]
pub struct Lifetime {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TyPathSegment {
    pub ident: Ident,
    pub generics: Option<Vec<GenericArg>>,
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub span: Span,
    pub src: Symbol,
}

#[derive(Debug, Clone)]
pub enum TraitObjectSyntax {
    Dyn,
    None,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Lit(Literal),
    Err,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Str(StrLit),
    Num(IntLit),
}

#[derive(Debug, Clone)]
pub struct IntLit {
    pub span: Span,
    pub src: Symbol,
    pub suffix: Option<Symbol>,
}
#[derive(Debug, Clone)]
pub struct Item {
    pub span: Span,
    pub id: NodeId,
    pub kind: ItemKind,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn(ItemFn),
    Err,
}

#[derive(Debug, Clone)]
pub struct ItemFn {
    pub unsafeness: Unsafeness,
    pub name: Ident,
    pub generics: Option<Vec<ItemGeneric>>,
    pub params: Vec<Pattern>,
    pub ret_ty: Option<Ty>,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub enum ItemGeneric {
    Named(Ident),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub span: Span,
    pub id: NodeId,
    pub kind: StmtKind,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let(StmtLet),
    Expr(Expr),
    Semi,
    Err,
}

#[derive(Debug, Clone)]
pub struct StmtLet {
    pub pat: Pattern,
    pub expr: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// An ident :
    /// ```rust
    ///     ident
    /// ```
    Ident(Ident),
    /// A type binding :
    /// ```rust
    ///     let ident : Ty = /*expr*/;
    /// ```
    TyBinding(Box<Self>, Box<Ty>),
    Err,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub items: Vec<Item>,
}
