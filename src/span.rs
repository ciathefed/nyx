use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl From<(usize, usize)> for Span {
    fn from(pair: (usize, usize)) -> Self {
        Span::new(pair.0, pair.1)
    }
}

impl From<Span> for SourceSpan {
    fn from(s: Span) -> Self {
        (s.start, s.end - s.start).into()
    }
}
