pub struct Flags {
    pub(crate) eq: bool,
    pub(crate) lt: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self {
            eq: false,
            lt: false,
        }
    }
}
