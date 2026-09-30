use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use colorful::Colorful;
use walkdir::WalkDir;

pub fn bless_tests(test_path: PathBuf, cwd: &Path) {
    let is_dir = test_path.is_dir();

    if is_dir {
        for res in WalkDir::new(&test_path) {
            let Ok(entry) = res else {
                continue;
            };
            let Some(extension) = entry.path().extension() else {
                continue;
            };
            if extension == "rs" || extension == "jessie" {
                run_test_rs(entry.path(), true, cwd);
            }
        }
    }
}

pub fn run_tests(test_path: PathBuf, cwd: &Path) {
    let is_dir = test_path.is_dir();

    if is_dir {
        for res in WalkDir::new(&test_path) {
            let Ok(entry) = res else {
                continue;
            };
            let Some(extension) = entry.path().extension() else {
                continue;
            };
            if extension == "rs" {
                run_test_rs(entry.path(), false, cwd);
            }
        }
    }
}

fn file_read_shebang(path: &Path) -> Option<String> {
    let file_contents = std::fs::read_to_string(path).expect("failed to read to string");
    let mut chars = file_contents.chars();
    if let Some(first) = chars.next()
        && let Some(second) = chars.next()
        && first == '#'
        && second == '!'
    {
        let mut out = String::new();
        while let Some(v) = chars.next()
            && v != '\n'
        {
            out.push(v);
        }
        Some(out)
    } else {
        None
    }
}

fn run_test_rs(path: &Path, bless: bool, cwd: &Path) {
    if let Some(shebang) = file_read_shebang(path) {
        check_command(
            path,
            Command::new("cargo")
                .arg("-q")
                .arg("run")
                .arg("--features")
                .arg("testing")
                .arg("--")
                .arg("--path")
                .arg(path)
                .arg("--")
                .arg(shebang.as_str().trim_start())
                .current_dir(cwd),
            bless,
        );
    }
}

fn check_command(path: &Path, command: &mut Command, bless: bool) {
    let output = command.output();
    if let Ok(v) = output {
        let command_output = match str::from_utf8(&v.stderr).ok() {
            Some(v) => v,
            _ => {
                println!("{} {path:#?}: invalid UTF8", "SKIPPING".rgb(255, 69, 0));
                return;
            }
        }
        .to_owned();

        let filtered_output = command_output.replace(&format!("{path:#?}"), "$PATH");

        if bless {
            let Ok(mut file) = std::fs::File::create(path.with_extension("stderr")) else {
                println!(
                    "{} {}",
                    "FAILED TO CREATE".red(),
                    path.with_extension("stderr").display().to_string().red()
                );
                return;
            };
            if file.write_all(&filtered_output.into_bytes()).is_err() {
                println!(
                    "{} {}",
                    "FAILED TO WRITE".red(),
                    path.with_extension("stderr").display().to_string().red()
                );
            }
        } else {
            let stderr_file = match std::fs::read_to_string(path.with_extension("stderr")).ok() {
                Some(v) => v,
                _ => {
                    println!(
                        "{} {path:#?}: no .stderr file available",
                        "SKIPPING".rgb(255, 69, 0)
                    );
                    return;
                }
            };

            if filtered_output == stderr_file {
                println!("{} {}", "OK : ".green(), path.display().to_string().green());
            } else {
                println!("{} {}", "FAILED : ".red(), path.display().to_string().red());
                std::process::exit(1)
            }
        }
    }
}
