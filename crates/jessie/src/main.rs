use clap::Parser;
use jessie_session::Session;
use jessie_span::SourceMap;

use crate::args::Args;

pub mod args;

fn main() {
    let args = Args::parse();

    let flags = args.flags();

    let sm = SourceMap::new();
    let mut sess = Session::new(sm);

    #[cfg(feature = "testing")]
    if flags.contains(&args::Flag::EmitTokens) {
        let (src, sp) = sess.sm.open(&args.path).unwrap_or_else(|_| {
            use colorful::Colorful;
            eprintln!("{}: failed to open path {:#?}", "ERROR".red(), args.path);
            std::process::exit(1);
        });
        let src = &src.to_string();
        let mut lexer = jessie_rust_lex::Lexer::new(src, sp, &mut sess);
        let mut out = vec![];
        loop {
            let tok = lexer.lex_token();
            if tok.is_eof() {
                break;
            }
            out.push(tok);
        }
        sess.report_errs();
        eprintln!("{out:#?}");
        std::process::exit(0);
    }
}
