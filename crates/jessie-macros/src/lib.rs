use std::path::PathBuf;

use proc_macro::{TokenStream as TS};

#[proc_macro]
pub fn init(stream: TS) -> TS {
    let first = stream.into_iter().next().unwrap(); // We need to get the first token inside the init macro to get the file it is invoked in.

    let file = first.span().file();

    let path: PathBuf = file.into();

    jessie_entry::entry(path).parse().expect("failed to parse compiler output")
}
