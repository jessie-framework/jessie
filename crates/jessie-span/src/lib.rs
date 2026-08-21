use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span(u32, u32);

impl Span {
    #[inline]
    pub const fn new(lo: u32, hi: u32) -> Self {
        Self(lo, hi)
    }

    #[inline]
    pub const fn nul() -> Self {
        Self(0, 0)
    }

    #[inline]
    pub const fn is_nul(self) -> bool {
        self.0 == 0 && self.1 == 0
    }

    #[inline]
    pub const fn lo(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn hi(self) -> u32 {
        self.1
    }

    #[inline]
    pub const fn contains(self, sp: Self) -> bool {
        self.lo() <= sp.lo() && self.hi() >= sp.hi()
    }
}

pub struct SourceMap {
    paths: Vec<(PathBuf, Span)>,
    src: String,
    last: usize,
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            paths: Vec::with_capacity(100),
            src: String::with_capacity(10000),
            last: 0,
        }
    }

    pub fn open(&mut self, path: &Path) -> std::io::Result<(&str, Span)> {
        for (pathb, sp) in &self.paths {
            if pathb == path {
                return Ok((self.span_str(*sp), *sp));
            }
        }
        let lo = self.src.len() as u32;
        let src = std::fs::read_to_string(path)?;
        self.src.push_str(&src);
        let hi = self.src.len() as u32;
        let sp = Span::new(lo, hi);
        self.paths.push((path.to_owned(), sp));
        Ok((self.span_str(sp), sp))
    }

    pub fn span_str(&self, sp: Span) -> &str {
        &self.src[sp.lo() as usize..sp.hi() as usize]
    }

    pub fn span_src(&mut self, sp: Span) -> PathBuf {
        if let Some((last_path, last_sp)) = self.paths.get(self.last)
            && last_sp.contains(sp)
        {
            return last_path.to_owned();
        }
        for (idx, (path, src_sp)) in self.paths.iter().enumerate() {
            if src_sp.contains(sp) {
                self.last = idx;
                return path.to_owned();
            }
        }
        unreachable!()
    }
}
