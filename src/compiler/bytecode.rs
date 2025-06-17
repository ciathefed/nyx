pub struct Bytecode {
    pub(crate) storage: Vec<u8>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn push<T: Into<u8>>(&mut self, value: T) {
        self.storage.push(value.into());
    }

    pub fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.storage.extend(iter);
    }

    pub fn write_u8_at(&mut self, offset: usize, value: u8) {
        self.storage[offset] = value;
    }

    pub fn write_u16_at(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.storage[offset..offset + 2].copy_from_slice(&bytes);
    }

    pub fn write_u32_at(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.storage[offset..offset + 4].copy_from_slice(&bytes);
    }

    pub fn write_u64_at(&mut self, offset: usize, value: u64) {
        let bytes = value.to_le_bytes();
        self.storage[offset..offset + 8].copy_from_slice(&bytes);
    }
}
