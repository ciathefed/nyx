#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Text,
    Data,
}

pub struct Bytecode {
    pub(crate) text: Vec<u8>,
    pub(crate) data: Vec<u8>,
}

impl Bytecode {
    pub fn new(capacity: Option<usize>) -> Self {
        let cap = capacity.unwrap_or(1024);
        Self {
            text: Vec::with_capacity(cap / 2),
            data: Vec::with_capacity(cap / 2),
        }
    }

    pub fn len(&self, section: Section) -> usize {
        match section {
            Section::Text => self.text.len(),
            Section::Data => self.data.len(),
        }
    }

    #[inline]
    pub fn push<T: Into<u8>>(&mut self, section: Section, value: T) {
        match section {
            Section::Text => self.text.push(value.into()),
            Section::Data => self.data.push(value.into()),
        }
    }

    #[inline]
    pub fn extend<T: IntoIterator<Item = u8>>(&mut self, section: Section, iter: T) {
        match section {
            Section::Text => self.text.extend(iter),
            Section::Data => self.data.extend(iter),
        }
    }

    #[inline]
    pub fn write_u8_at(&mut self, section: Section, offset: usize, value: u8) {
        match section {
            Section::Text => self.text[offset] = value,
            Section::Data => self.data[offset] = value,
        }
    }

    #[inline]
    pub fn write_u16_at(&mut self, section: Section, offset: usize, value: u16) {
        match section {
            Section::Text => self.text[offset..offset + 2].copy_from_slice(&value.to_le_bytes()),
            Section::Data => self.data[offset..offset + 2].copy_from_slice(&value.to_le_bytes()),
        }
    }

    #[inline]
    pub fn write_u32_at(&mut self, section: Section, offset: usize, value: u32) {
        match section {
            Section::Text => self.text[offset..offset + 4].copy_from_slice(&value.to_le_bytes()),
            Section::Data => self.data[offset..offset + 4].copy_from_slice(&value.to_le_bytes()),
        }
    }

    #[inline]
    pub fn write_u64_at(&mut self, section: Section, offset: usize, value: u64) {
        match section {
            Section::Text => self.text[offset..offset + 8].copy_from_slice(&value.to_le_bytes()),
            Section::Data => self.data[offset..offset + 8].copy_from_slice(&value.to_le_bytes()),
        }
    }

    pub fn finalize(&mut self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(&self.text);
        bytes.extend(&self.data);
        bytes
    }
}
