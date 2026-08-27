//! This crate gets built and run to run internal Jessie related tests.

use std::path::PathBuf;

use clap::Parser;

#[path = "."]
pub mod subcommand {

    #[path = "./test.rs"]
    pub mod test;
}

fn main() {
    let args = Args::parse();

    match args.sub {
        Sub::Test { path, bless, cwd } => {
            if bless {
                subcommand::test::bless_tests(path, &cwd);
            } else {
                subcommand::test::run_tests(path, &cwd);
            }
        }
    }
}

#[derive(clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub sub: Sub,
}

#[derive(clap::Subcommand, Clone)]
pub enum Sub {
    Test {
        path: PathBuf,
        cwd: PathBuf,
        #[arg(short, long)]
        bless: bool,
    },
}
