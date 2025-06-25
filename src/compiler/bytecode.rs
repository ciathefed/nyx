pub struct Bytecode {
    pub(crate) storage: Vec<u8>,
}

impl Bytecode {
    pub fn new(capacity: Option<usize>) -> Self {
        let storage = Vec::with_capacity(capacity.unwrap_or(1024));
        Self { storage }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    #[inline]
    pub fn push<T: Into<u8>>(&mut self, value: T) {
        self.storage.push(value.into());
    }

    #[inline]
    pub fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.storage.extend(iter);
    }

    #[inline]
    pub fn write_u8_at(&mut self, offset: usize, value: u8) {
        self.storage[offset] = value;
    }

    #[inline]
    pub fn write_u16_at(&mut self, offset: usize, value: u16) {
        self.storage[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn write_u32_at(&mut self, offset: usize, value: u32) {
        self.storage[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[inline]
    pub fn write_u64_at(&mut self, offset: usize, value: u64) {
        self.storage[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}
