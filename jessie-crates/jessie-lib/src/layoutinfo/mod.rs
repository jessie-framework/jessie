///This struct gives info to Jessie for where to place its components.
pub struct LayoutInfo;

impl LayoutInfo {
    ///This function is for placing static components on the screen.
    ///Static components are components that dont have any particular state that they have to follow, for example a Colored rectangle, etc.
    pub fn put(&mut self) {}

    ///This function is for putting expectations on the screen.
    /// Expectations are "mini programs" that communicate with the "mini program" that placed them.
    pub fn expect(&mut self) {}
}
