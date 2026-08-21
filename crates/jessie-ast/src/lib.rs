use jessie_span::Span;
use jessie_tokenizer::TokenKind;

pub mod krate;
pub mod parser;
mod passes;
mod tokenizer;

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    CouldntParse,
    NoFilePrefix,
    CouldntOpenFile,
    UnexpectedValue,
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
    ClosedElementThatWasntOpened {
        sp: Span,
    },
    ProgramAlreadyHasParameter {
        sp: Span,
    },
    MissingProgramParameters {
        sp: Span,
    },
    NumberConvertError {
        sp: Span,
    },
    UnrecognizedProgramField {
        sp: Span,
    },
}
