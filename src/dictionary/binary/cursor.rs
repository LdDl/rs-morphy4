use crate::dictionary::binary::error::BinaryError;

pub struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn take(&mut self, length: usize) -> Result<&'a [u8], BinaryError> {
        let remaining = self.bytes.len() - self.offset;
        if length > remaining {
            return Err(BinaryError::UnexpectedEnd {
                offset: self.offset,
                needed: length,
                remaining,
            });
        }
        let result = &self.bytes[self.offset..self.offset + length];
        self.offset += length;
        Ok(result)
    }

    pub fn u16(&mut self) -> Result<u16, BinaryError> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn u32(&mut self) -> Result<u32, BinaryError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn finish(self) -> Result<(), BinaryError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining != 0 {
            return Err(BinaryError::TrailingData {
                offset: self.offset,
                remaining,
            });
        }
        Ok(())
    }
}
