use crate::layoutinfo::LayoutInfo;
use std::any::Any;

pub trait Expectation<Message>
where
    Message: Any,
{
    fn init_state() -> Self;

    fn build(&mut self, layout: &mut LayoutInfo);
}
