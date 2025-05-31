use crate::appinfo::AppInfo;
use crate::layoutinfo::LayoutInfo;
use crate::layoutprovider::LayoutProvider;
pub mod appinfo;
pub mod expectation;
pub mod layoutinfo;
pub mod layoutprovider;

pub fn run(app: AppInfo, mainexpectation: impl LayoutProvider) {}
