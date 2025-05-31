use jessie_lib::layoutprovider::LayoutProvider;

struct ExampleState {
    count: u32,
}

impl LayoutProvider for ExampleState {
    fn build() {}
}

#[jessie_lib::app]
fn app() -> impl LayoutProvider {
    ExampleState { count: 0 }
}
