//! ROM access adapter for report generation

/// Trait for ROM access in report generation
pub trait RomAccess {
    fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], String>;
}

/// Adapter for Rom struct from rom-core
pub struct RomAdapter<'a> {
    pub data: &'a [u8],
}

impl<'a> RomAdapter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> RomAccess for RomAdapter<'a> {
    fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], String> {
        if offset + len > self.data.len() {
            return Err(format!(
                "Offset {} + len {} exceeds ROM size {}",
                offset,
                len,
                self.data.len()
            ));
        }
        Ok(&self.data[offset..offset + len])
    }
}
