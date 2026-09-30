use std::path::PathBuf;

use colorful::Colorful;

#[derive(clap::Parser)]
pub struct Args {
    #[arg(long)]
    pub path: PathBuf,

    pub flags: Vec<String>,
}

impl Args {
    pub fn flags(&self) -> Vec<Flag> {
        let mut out = vec![];
        for v in &self.flags {
            match v.as_str() {
                #[cfg(feature = "testing")]
                "--test=emit-tokens" => out.push(Flag::EmitTokens),
                #[cfg(feature = "testing")]
                "--test=recreate" => out.push(Flag::Recreate),
                #[cfg(feature = "testing")]
                "--test=emit-ast" => out.push(Flag::EmitAst),
                err => {
                    println!("{}: unexpected flag {err}", "ERROR".red(),);
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    #[cfg(feature = "testing")]
    EmitTokens,
    #[cfg(feature = "testing")]
    Recreate,
    #[cfg(feature = "testing")]
    EmitAst,
}
