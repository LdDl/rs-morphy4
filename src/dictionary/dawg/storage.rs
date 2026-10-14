use crate::dictionary::binary::cursor::Cursor;
use crate::dictionary::binary::error::BinaryError;

#[derive(Debug)]
pub struct Storage {
    pub units: Vec<u32>,
    pub guide: Vec<[u8; 2]>,
}

impl Storage {
    pub fn decode(bytes: &[u8]) -> Result<Self, BinaryError> {
        let mut cursor = Cursor::new(bytes);
        let count = cursor.u32()? as usize;
        if count == 0 {
            return Err(BinaryError::EmptyDawg);
        }
        // Division checks the file size before multiplication and allocation.
        if count > bytes.len().saturating_sub(4) / 4 {
            return Err(BinaryError::CountMismatch {
                expected: count,
                actual: bytes.len().saturating_sub(4) / 4,
            });
        }
        let units = cursor
            .take(count * 4)?
            .as_chunks::<4>()
            .0
            .iter()
            .copied()
            .map(u32::from_le_bytes)
            .collect();
        let guide_count = cursor.u32()? as usize;
        if guide_count != count {
            return Err(BinaryError::CountMismatch {
                expected: count,
                actual: guide_count,
            });
        }
        let guide = cursor.take(guide_count * 2)?.as_chunks::<2>().0.to_vec();
        cursor.finish()?;
        Ok(Self { units, guide })
    }
}
