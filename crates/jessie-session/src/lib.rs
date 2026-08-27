use std::path::Path;

use jessie_span::SourceMap;

pub struct Session {
    pub sm: SourceMap,
    msg: String,
    count: u32,
    erroneous: bool,
}

impl Session {
    pub fn new(sm: SourceMap) -> Self {
        Self {
            sm,
            msg: String::new(),
            count: 0,
            erroneous: false,
        }
    }

    pub fn new_did(&mut self) -> DefId {
        let out = DefId(self.count);
        self.count += 1;
        out
    }

    pub fn new_err_sp(&mut self, sp: jessie_span::Span, msg: &str) {
        self.erroneous = true;
        self.msg.push_str(&format!(
            "
                ERROR : {}
                    -> at {:#?} line {} row {}
                ```
                   {} 
                ```
            ",
            msg,
            self.sm.span_src(sp),
            self.sm.span_loc(sp).0,
            self.sm.span_loc(sp).1,
            self.sm.span_str(sp)
        ));
    }

    pub fn new_err_path(&mut self, path: &Path, msg: &str) {
        self.msg.push_str(&format!(
            "
                ERROR : {}
                    -> at {:#?}
            ",
            msg, path
        ));
    }

    pub fn new_diag(&mut self, diag: impl Diagnostic) {
        diag.diag(self);
    }

    pub fn report_errs(&mut self) {
        if !self.msg.is_empty() {
            eprintln!("{}", self.msg);
            self.msg.clear();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(u32);

impl DefId {
    #[inline]
    pub const fn err() -> Self {
        Self(u32::MAX)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

pub trait Diagnostic {
    fn diag(self, sess: &mut Session);
}
