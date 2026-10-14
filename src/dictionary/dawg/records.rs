use crate::dictionary::binary::error::BinaryError;

mod sealed {
    pub trait Sealed {}
}

/// A fixed-size, big-endian record stored as a base64 DAWG payload.
pub trait DawgRecord: sealed::Sealed + Sized {
    const SIZE: usize;
    fn decode(bytes: &[u8]) -> Result<Self, BinaryError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordRecord {
    pub paradigm_id: u16,
    pub form_index: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictionRecord {
    pub count: u32,
    pub paradigm_id: u16,
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
