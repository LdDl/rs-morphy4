use crate::dictionary::binary::error::BinaryError;

mod sealed {
    pub trait Sealed {}
}

/// A fixed-size, big-endian record stored as a base64 DAWG payload.
pub trait DawgRecord: sealed::Sealed + Sized {
    /// Decoded payload size in bytes.
    const SIZE: usize;
    /// Decodes exactly SIZE bytes or returns a record-length error.
    fn decode(bytes: &[u8]) -> Result<Self, BinaryError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Dictionary reference encoded as two big-endian u16 values.
pub struct WordRecord {
    /// Index into the paradigm table.
    pub paradigm_id: u16,
    /// Index of the form within the paradigm.
    pub form_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Suffix prediction count and dictionary reference encoded as u32, u16, u16.
pub struct PredictionRecord {
    /// Observed frequency recorded by the dictionary compiler.
    pub count: u32,
    /// Index into the paradigm table.
    pub paradigm_id: u16,
    /// Index of the form within the paradigm.
    pub form_index: u16,
}

impl sealed::Sealed for WordRecord {}
impl sealed::Sealed for PredictionRecord {}

impl DawgRecord for WordRecord {
    const SIZE: usize = 4;

    fn decode(bytes: &[u8]) -> Result<Self, BinaryError> {
        check_length(bytes, Self::SIZE)?;
        Ok(Self {
            paradigm_id: u16::from_be_bytes([bytes[0], bytes[1]]),
            form_index: u16::from_be_bytes([bytes[2], bytes[3]]),
        })
    }
}

impl DawgRecord for PredictionRecord {
    const SIZE: usize = 8;

    fn decode(bytes: &[u8]) -> Result<Self, BinaryError> {
        check_length(bytes, Self::SIZE)?;
        Ok(Self {
            count: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            paradigm_id: u16::from_be_bytes([bytes[4], bytes[5]]),
            form_index: u16::from_be_bytes([bytes[6], bytes[7]]),
        })
    }
}

fn check_length(bytes: &[u8], expected: usize) -> Result<(), BinaryError> {
    if bytes.len() != expected {
        return Err(BinaryError::InvalidRecordLength {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}
