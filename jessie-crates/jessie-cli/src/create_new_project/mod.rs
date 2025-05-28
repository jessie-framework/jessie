use std::fs::File;
use std::fs::write;
use std::path::Path;
use std::process::Command;

pub fn create_new_project(name: &str) {
    Command::new("cargo")
        .arg("new")
        .arg(name)
        .status()
        .expect("failed to create new library");
    Command::new("cargo")
        .arg("add")
        .arg("jessie-lib")
        .current_dir(Path::new(name))
        .status()
        .expect("failed to add jessie-lib to the newly created crate");
    Command::new("cargo")
        .arg("add")
        .arg("jessie-build")
        .current_dir(Path::new(name))
        .status()
        .expect("failed to add jessie-build to the newly created crate");
    create_file_at_dir(name, "config.ron").expect("failed to create config.ron");
    create_file_at_dir(name, "build.rs").expect("failed to create config.ron");
    write(
        Path::new(name).join("build.rs"),
        include_str!("buildtemplate.rs"),
    )
    .expect("failed to write to build.rs");
}

fn create_file_at_dir(projectname: &str, filename: &str) -> std::io::Result<File> {
    let new_file = Path::new(projectname).join(filename);
    let file = File::create(new_file)?;
    Ok(file)
}
