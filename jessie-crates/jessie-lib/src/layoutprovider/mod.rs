///The root of a Jessie app.
///Jessie programs are "mini programs communicating with each other".
///In our case, we use this LayoutProvider struct to tell Jessie that this is our "main mini program".
pub trait LayoutProvider {
    fn build();
}
