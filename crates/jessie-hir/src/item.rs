use jessie_span::Span;

pub struct Crate<'hir> {
    pub name: &'hir str,
    pub module: Module<'hir>,
}

#[derive(Debug, Clone, Copy)]
pub struct Module<'hir> {
    pub name: &'hir str,
    pub children: &'hir [Self],
    pub document: Option<Document<'hir>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Document<'hir> {
    pub items: &'hir [Item<'hir>],
}

#[derive(Debug, Clone, Copy)]
pub struct Item<'hir> {
    pub kind: ItemKind<'hir>,
}

#[derive(Debug, Clone, Copy)]
pub enum ItemKind<'hir> {
    Entry(EntryBlock<'hir>),
    Extends(ExtendsBlock<'hir>),
    Program(ProgramBlock),
}

#[derive(Debug, Clone, Copy)]
pub struct ProgramBlock {
    pub name: Ident,
    pub vs: ShaderId,
    pub fs: ShaderId,
}

#[derive(Debug, Clone, Copy)]
pub struct ShaderId(u32);

#[derive(Debug, Clone, Copy)]
pub struct EntryBlock<'hir> {
    pub name: Ident,
    pub tree: Tree<'hir>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct ExtendsBlock<'hir> {
    pub name: Ident,
    pub tree: Tree<'hir>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Tree<'hir> {
    pub elements: &'hir [Element<'hir>],
}

#[derive(Debug, Clone, Copy)]
pub enum Element<'hir> {
    Nested(NestedElement<'hir>),
    Single(SingleElement<'hir>),
    Text(TextElement<'hir>),
}

#[derive(Debug, Clone, Copy)]
pub struct SingleElement<'hir> {
    pub item: Item<'hir>,
    pub props: &'hir [Prop],
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct TextElement<'hir> {
    pub params: &'hir [FormatTextParam<'hir>],
}

#[derive(Debug, Clone, Copy)]
pub struct FormatTextParam<'hir> {
    pub is_formatted: bool,
    pub text: &'hir str,
}

#[derive(Debug, Clone, Copy)]
pub struct NestedElement<'hir> {
    pub item: Item<'hir>,
    pub span: Span,
    pub children: &'hir [Element<'hir>],
    pub props: &'hir [Prop],
}

#[derive(Debug, Clone, Copy)]
pub enum Prop {
    Single(Span, Ident),
    NotSingle { prop_name: Ident, prop_val: Ident },
}

#[derive(Debug, Clone, Copy)]
pub struct Ident(Span);

#[derive(Debug, Clone, Copy)]
pub struct HirId(u32);
