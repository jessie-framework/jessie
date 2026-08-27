use std::path::PathBuf;

use jessie_session::Diagnostic;
use jessie_span::Span;
use jessie_tokenizer::TokenKind;

pub mod krate;
pub mod parser;
mod passes;
mod tokenizer;

#[derive(Debug, Clone)]
pub enum ParseError {
    CouldntParse {
        path: PathBuf,
    },
    NoFilePrefix {
        path: PathBuf,
    },
    CouldntOpenFile {
        path: PathBuf,
    },
    UnexpectedValue {
        expected: String,
        found: String,
        sp: Span,
    },
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        sp: Span,
    },
    ClosedElementThatWasntOpened {
        sp: Span,
    },
    ProgramAlreadyHasParameter {
        sp: Span,
        param: &'static str,
    },
    MissingProgramParameters {
        sp: Span,
        missing_params: Vec<&'static str>,
    },
    NumberConvertError {
        sp: Span,
    },
    UnrecognizedProgramField {
        sp: Span,
    },
}

impl Diagnostic for ParseError {
    fn diag(self, sess: &mut jessie_session::Session) {
        match self {
            Self::CouldntParse { path } => sess.new_err_path(&path, "couldn't parse .rs file"),
            Self::NoFilePrefix { path } => sess.new_err_path(&path, "no file prefix"),
            Self::CouldntOpenFile { path } => sess.new_err_path(&path, "couldn't open file"),
            Self::UnexpectedValue {
                expected,
                found,
                sp,
            } => sess.new_err_sp(sp, &format!("expected {expected}, found {found}")),
            Self::UnexpectedToken {
                expected,
                found,
                sp,
            } => sess.new_err_sp(
                sp,
                &format!("expected {}, found {}", expected.as_str(), found.as_str()),
            ),
            Self::ClosedElementThatWasntOpened { sp } => {
                sess.new_err_sp(sp, "closed element that wasn't opened")
            }
            Self::ProgramAlreadyHasParameter { sp, param } => {
                sess.new_err_sp(sp, &format!("program already has parameter {param}"))
            }
            Self::MissingProgramParameters { sp, missing_params } => sess.new_err_sp(
                sp,
                &format!(
                    "missing program parameters \"{}\"",
                    missing_params.join(",")
                ),
            ),
            Self::NumberConvertError { sp } => sess.new_err_sp(sp, "number convert error"),
            Self::UnrecognizedProgramField { sp } => {
                sess.new_err_sp(sp, "unrecognized program field")
            }
        }
    }
}
