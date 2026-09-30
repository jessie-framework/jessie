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
        let mut toks = vec![];
        loop {
            let tok = lexer.lex_token();
            if tok.is_eof() {
                break;
            }
            toks.push(tok);
        }

        for tok in toks {
            eprintln!(
                "{} [({})@{}]",
                sess.sm.span_str(tok.span),
                tok.kind,
                tok.span
            );
        }
        sess.report_errs();
        std::process::exit(0);
    }

    #[cfg(feature = "testing")]
    if flags.contains(&args::Flag::Recreate) {
        let (src, sp) = sess.sm.open(&args.path).unwrap_or_else(|_| {
            use colorful::Colorful;
            eprintln!("{}: failed to open path {:#?}", "ERROR".red(), args.path);
            std::process::exit(1);
        });
        let src = &src.to_string();
        let mut lexer = jessie_rust_lex::Lexer::new(src, sp, &mut sess);
        let mut toks = vec![];
        loop {
            let tok = lexer.lex_token();
            if tok.is_eof() || (tok.span.lo() == sp.lo() && tok.span.hi() == sp.hi()) {
                break;
            }
            toks.push(tok);
        }
        let mut out = String::new();
        for tok in toks {
            out.push_str(sess.sm.span_str(tok.span));
            out.push('\n');
        }
        sess.report_errs();
        eprintln!("{out}");
        std::process::exit(0);
    }

    #[cfg(feature = "testing")]
    if flags.contains(&args::Flag::EmitAst) {
        use jessie_ast::{TokenStream, parser::Parser};

        let (src, sp) = sess.sm.open(&args.path).unwrap_or_else(|_| {
            use colorful::Colorful;
            eprintln!("{}: failed to open path {:#?}", "ERROR".red(), args.path);
            std::process::exit(1);
        });
        let src = &src.to_string();
        let lexer = jessie_rust_lex::Lexer::new(src, sp, &mut sess);
        let ts = TokenStream::new(lexer);
        let mut parser = Parser::new(ts, &mut sess);
        let doc = parser.parse_document();

        sess.report_errs();
        eprintln!("{doc:#?}");
        std::process::exit(0);
    }
}
