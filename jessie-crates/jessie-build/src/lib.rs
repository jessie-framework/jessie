use std::path::Path;
pub fn build() {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("jessie-build : error finding environment variable CARGO_MANIFEST_DIR");
    let config_ron_dir = Path::new(&manifest_dir).join("config.ron");
}
