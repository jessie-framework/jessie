//! This crate is used in the `jessie::init!(_)` macro to parse and download dependencies that depend on jessie into a temporary `.jessie` directory so files with `jessie` extensions can get parsed across crates.

use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{Cursor, Read, Write},
    path::PathBuf,
};

use flate2::read::GzDecoder;
use reqwest::header::USER_AGENT;
use semver::{Version, VersionReq};
use tar::Archive;
use toml::{Value, map::Map};

/// The entry to using the crate. This gets invoked inside of the `jessie::init!(_)` macro.
pub fn entry() -> Dependency {
    // Find and parse Cargo.toml
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set")
        .into();

    let cargo_toml_dir = manifest_dir.join("Cargo.toml");

    let contents = std::fs::read_to_string(cargo_toml_dir).expect("failed to read Cargo.toml");

    let toml_table: toml::Table = contents.parse().expect("failed to parse Cargo.toml");

    // Get package name and version
    let package_name = toml_table["package"]["name"].as_str().unwrap().to_owned();

    let package_version = toml_table["package"]["version"]
        .as_str()
        .unwrap()
        .to_owned();

    let dependencies = toml_table
        .get("dependencies")
        .map(|x| x.as_table().unwrap());

    let dep_dir = get_cargo_manifest_dir();

    let mut dep_cache = load_dep_cache();

    let deps = get_dependencies(dependencies, dep_dir, &mut dep_cache);

    let is_binary = std::fs::exists(manifest_dir.join("src").join("main.rs")).unwrap();

    let entry = if is_binary {
        manifest_dir.join("src").join("main.rs")
    } else {
        manifest_dir.join("src").join("lib.rs")
    };

    Dependency {
        version: package_version,
        name: package_name,
        path: manifest_dir,
        is_binary,
        entry,
        dependencies: deps,
    }
}

/// A dependency in Cargo.toml whose information was already inquired from crates.io , to not make extra requests
#[derive(jessie_thing::Serialize, jessie_thing::Deserialize, PartialEq, Eq, Hash)]
pub(crate) struct CachedDependencyOnline {
    pub name: String,
    pub version: String,
}

/// Loads the cached dependencies.
fn load_dep_cache() -> HashSet<CachedDependencyOnline> {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set")
        .into();

    let path = manifest_dir.join(".jessie").join("depcache");

    if !std::fs::exists(&path).expect("failed to read path") {
        std::fs::create_dir_all(manifest_dir.join(".jessie"))
            .expect("failed to create .jessie dir");
        std::fs::write(&path, []).expect("failed to create file");
    }

    let contents = std::fs::read(path).expect("failed to read contents");

    let deser: Vec<CachedDependencyOnline> =
        jessie_thing::from_bytes(&contents).expect("failed to deserialize");

    let mut out = HashSet::new();

    for dep in deser {
        out.insert(dep);
    }

    out
}

/// Writes a cached dependency into the dependency cache.
fn write_cached_dependency(
    dep_cache: &mut HashSet<CachedDependencyOnline>,
    name: String,
    version: String,
) {
    dep_cache.insert(CachedDependencyOnline {
        name: name.clone(),
        version: version.clone(),
    });
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set")
        .into();

    let path = manifest_dir.join(".jessie").join("depcache");

    let ser = jessie_thing::into_vec(CachedDependencyOnline { name, version });

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect("failed to open file");

    file.write_all(&ser)
        .expect("failed to write cached dependency");
}

fn get_cargo_manifest_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set")
        .into()
}

fn get_jessie_dep_dir() -> PathBuf {
    let target_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set")
        .into();

    target_dir.join(".jessie").join("deps")
}

#[derive(Debug, Clone)]
/// A dependency.
pub struct Dependency {
    /// The version of the dependency.
    pub version: String,
    /// The name of the dependency.
    pub name: String,
    /// The root path of the dependency.
    pub path: PathBuf,
    /// The root file of the dependency.
    pub entry: PathBuf,
    /// Whether or not the crate is a binary crate.
    pub is_binary: bool,
    pub dependencies: Vec<Self>,
}

/// Downloads all dependencies from the [dependencies] section of Cargo.toml and their related dependencies.
fn get_dependencies(
    values: Option<&Map<String, Value>>,
    dep_dir: PathBuf,
    dep_cache: &mut HashSet<CachedDependencyOnline>,
) -> Vec<Dependency> {
    let mut out = vec![];
    if let Some(values) = values {
        for (dep_name, value) in values {
            if let Some(krate) =
                extract_crate(dep_dir.clone(), dep_name.to_owned(), value, dep_cache)
            {
                out.push(krate);
            }
        }
    }
    out
}

fn toml_parse_dependencies(path: PathBuf) -> Option<Map<String, Value>> {
    if let Ok(contents) = std::fs::read_to_string(&path) {
        let toml_table: toml::Table = contents.parse().expect("failed to parse Cargo.toml");

        toml_table
            .get("dependencies")
            .map(|x| x.as_table().unwrap())
            .cloned()
            .filter(|v| v.contains_key("jessie"))
    } else {
        None
    }
}

fn toml_parse_version(path: PathBuf) -> String {
    let contents = std::fs::read_to_string(path).expect("failed to read Cargo.toml");

    let toml_table: toml::Table = contents.parse().expect("failed to parse Cargo.toml");

    toml_table["package"]
        .as_table()
        .expect("failed to get dependencies")
        .clone()["version"]
        .to_string()
}

fn extract_crate(
    dep_dir: PathBuf,
    dep_name: String,
    dep_value: &Value,
    dep_cache: &mut HashSet<CachedDependencyOnline>,
) -> Option<Dependency> {
    match dep_value {
        // A string value. Looks something like `dependency = "0.1.1"`
        Value::String(predicate) => download_crate(dep_name, predicate, dep_cache),
        Value::Table(table) => {
            if let Some(Some(git_repo_url)) = table.get("git").map(|x| x.as_str()) {
                let dep_dir = get_jessie_dep_dir();

                git2::Repository::clone(
                    git_repo_url,
                    dep_dir.join(format!("{}@{git_repo_url}", dep_name)),
                )
                .expect("failed to clone git repository");

                let repo_toml_dir = dep_dir.join(&dep_name).join("Cargo.toml");

                let dependencies = toml_parse_dependencies(repo_toml_dir.clone());

                let version = toml_parse_version(repo_toml_dir);

                let is_binary =
                    std::fs::exists(dep_dir.join(format!("{}@{git_repo_url}", dep_name))).unwrap();

                let entry = if is_binary {
                    dep_dir
                        .join(format!("{}@{git_repo_url}", dep_name))
                        .join("src")
                        .join("main.rs")
                } else {
                    dep_dir
                        .join(format!("{}@{git_repo_url}", dep_name))
                        .join("src")
                        .join("lib.rs")
                };

                return Some(Dependency {
                    version,
                    name: dep_name.clone(),
                    path: dep_dir.join(format!("{}@{git_repo_url}", dep_name)),
                    dependencies: get_dependencies(dependencies.as_ref(), dep_dir, dep_cache),
                    entry,
                    is_binary,
                });
            } else if let Some(Some(file_path)) = table.get("path").map(|x| x.as_str()) {
                let toml_dir = dep_dir.join(file_path).join("Cargo.toml");

                let dependencies = toml_parse_dependencies(toml_dir.clone());

                let version = toml_parse_version(toml_dir);

                let is_binary =
                    std::fs::exists(dep_dir.join(file_path).join("src").join("main.rs")).unwrap();

                let entry = if is_binary {
                    dep_dir.join(file_path).join("src").join("main.rs")
                } else {
                    dep_dir.join(file_path).join("src").join("lib.rs")
                };

                return Some(Dependency {
                    is_binary,
                    entry,
                    version,
                    name: dep_name.clone(),
                    path: dep_dir.clone().join(file_path),
                    dependencies: get_dependencies(
                        dependencies.as_ref(),
                        dep_dir.join(file_path),
                        dep_cache,
                    ),
                });
            } else if let Some(Some(predicate)) = table.get("version").map(|x| x.as_str()) {
                return download_crate(dep_name, predicate, dep_cache);
            }
            None
        }
        _ => None,
    }
}

fn download_crate(
    dep_name: String,
    predicate: &str,
    dep_cache: &mut HashSet<CachedDependencyOnline>,
) -> Option<Dependency> {
    if dep_cache.contains(&CachedDependencyOnline {
        name: dep_name.clone(),
        version: predicate.to_string(),
    }) {
        return None;
    }

    write_cached_dependency(dep_cache, dep_name.clone(), predicate.to_string());

    let req = VersionReq::parse(predicate).expect("failed to parse predicate");

    let dep_dir = get_jessie_dep_dir();

    let mut versions_json = String::new();

    let client = reqwest::blocking::Client::new();

    client
        .get(format!(
            "https://crates.io/api/v1/crates/{dep_name}/versions"
        ))
        .header(USER_AGENT, "jessie")
        .send()
        .expect("failed to get response")
        .read_to_string(&mut versions_json)
        .expect("failed to write json");

    let versions_table: serde_json::Value = versions_json.parse().expect("failed to parse json");

    let versions = versions_table["versions"].as_array()?;

    let best_match = versions
        .iter()
        .find(|val| {
            let num = val["num"].as_str().expect("not a str");
            let version = Version::parse(num).expect("not a valid version");
            req.matches(&version)
        })
        .expect("failed to find a good match")["num"]
        .as_str()
        .expect("best match not a str");

    let dependencies_link = format!(
        "https://crates.io{}",
        versions
            .iter()
            .find(|val| {
                let num = val["num"].as_str().expect("not a str");
                let version = Version::parse(num).expect("not a valid version");
                req.matches(&version)
            })
            .expect("failed to find a good match")["links"]["dependencies"]
            .as_str()
            .expect("dependencies link not a str")
    );

    if !std::fs::exists(dep_dir.join(format!("{dep_name}-{best_match}"))).expect("failed to read") {
        let mut dependencies_json = String::new();

        client
            .get(dependencies_link)
            .header(USER_AGENT, "jessie")
            .send()
            .expect("failed to get response")
            .read_to_string(&mut dependencies_json)
            .expect("failed to write json");

        let dependencies = dependencies_json
            .parse::<serde_json::Value>()
            .expect("failed to parse json")["dependencies"]
            .as_array()
            .cloned();

        if let Some(array) = dependencies
            && array
                .iter()
                .find(|val| {
                    if let Some(crate_id) = val["crate_id"].as_str()
                        && crate_id == "jessie"
                    {
                        true
                    } else {
                        false
                    }
                })
                .is_none()
        {
            return None;
        }

        std::fs::create_dir_all(dep_dir.join(format!("{dep_name}-{best_match}")))
            .expect("failed to create directory");

        let crate_bytes = reqwest::blocking::get(format!(
            "https://crates.io/api/v1/crates/{dep_name}/{best_match}/download"
        ))
        .expect("failed to get response")
        .bytes()
        .expect("failed to get response bytes");

        let cursor = Cursor::new(crate_bytes);

        let decoder = GzDecoder::new(cursor.clone());

        let mut archive = Archive::new(decoder);

        archive.unpack(&dep_dir).expect("failed to unpack archive");
    }
    let archive_toml_dir = dep_dir
        .join(format!("{dep_name}-{best_match}"))
        .join("Cargo.toml");

    let dependencies = toml_parse_dependencies(archive_toml_dir);

    let is_binary = std::fs::exists(
        dep_dir
            .join(format!("{dep_name}-{best_match}"))
            .join("src")
            .join("main.rs"),
    )
    .unwrap();

    let entry = if is_binary {
        dep_dir
            .join(format!("{dep_name}-{best_match}"))
            .join("src")
            .join("main.rs")
    } else {
        dep_dir
            .join(format!("{dep_name}-{best_match}"))
            .join("src")
            .join("lib.rs")
    };

    Some(Dependency {
        entry,
        is_binary,
        version: best_match.to_owned(),
        name: dep_name.clone(),
        path: dep_dir.join(format!("{dep_name}-{best_match}")),
        dependencies: get_dependencies(dependencies.as_ref(), dep_dir, dep_cache),
    })
}
