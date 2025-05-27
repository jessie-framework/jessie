use jessie_lib::appinfo::AppInfo;
use std::any::Any;
use std::fs;
use std::path::Path;
pub fn build() {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("jessie-build : error finding environment variable CARGO_MANIFEST_DIR");
    let config_ron_dir = Path::new(&manifest_dir).join("config.ron");
    let config_ron_str =
        fs::read_to_string(config_ron_dir).expect("jessie-build : error reading config.ron file");

    let generated: AppInfo =
        ron::from_str(&config_ron_str).expect("jessie-build : error generating ron file");

    let generated_str = format!(include_str!("generated.rs"), generated);

    let out_dir = std::env::var_os("OUT_DIR")
        .expect("jessie-build : error finding environment variable OUT_DIR");

    let out_path = Path::new(&out_dir).join("appinfo.rs");

    fs::write(out_path, generated_str);
}
