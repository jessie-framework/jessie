use std::fs::File;
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
    create_file_at_dir(name, "config.ron").expect("failed to create config.ron");
}

fn create_file_at_dir(projectname: &str, filename: &str) -> std::io::Result<File> {
    let new_file = Path::new(projectname).join(filename);
    let file = File::create(new_file)?;
    Ok(file)
}
