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

impl Into<Span> for (usize, usize) {
    fn into(self) -> Span {
        Span::new(self.0, self.1)
    }
}
