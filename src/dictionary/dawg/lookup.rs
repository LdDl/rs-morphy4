use base64::{Engine, engine::general_purpose::STANDARD};

use crate::dictionary::binary::error::BinaryError;
use crate::dictionary::dawg::records::DawgRecord;
use crate::dictionary::dawg::storage::Storage;
use crate::dictionary::dawg::units;

impl Storage {
    pub fn lookup<R: DawgRecord>(&self, key: &str) -> Result<Vec<R>, BinaryError> {
        // Zero and one are reserved for terminal and payload transitions.
        if key.bytes().any(|b| b <= 1) {
            return Ok(Vec::new());
        }
        let mut index = 0;
        for label in key.bytes().chain(std::iter::once(1)) {
            match units::follow(&self.units, index, label)? {
                Some(next) => index = next,
                None => return Ok(Vec::new()),
            }
        }
        let mut results = Vec::new();
        self.collect(index, &mut Vec::new(), &mut results)?;
        Ok(results)
    }

    fn collect<R: DawgRecord>(
        &self,
        index: usize,
        payload: &mut Vec<u8>,
        results: &mut Vec<R>,
    ) -> Result<(), BinaryError> {
        // The compiled format permits a final newline after padded base64.
        let limit = R::SIZE.div_ceil(3) * 4 + 1;
        if payload.len() > limit {
            return Err(BinaryError::PayloadTooLong { limit });
        }
        let terminal = units::has_leaf(units::unit(&self.units, index)?);
        if terminal {
            units::validate_leaf(&self.units, index)?;
            let encoded = payload.strip_suffix(b"\n").unwrap_or(payload);
            let decoded = STANDARD
                .decode(encoded)
                .map_err(|source| BinaryError::InvalidBase64 { source })?;
            results.push(R::decode(&decoded)?);
        }
        let mut label = self.guide[index][0];
        if !terminal && label == 0 {
            return Err(BinaryError::MissingTerminal { index });
        }
        let mut seen = [false; 256];
        while label != 0 {
            if seen[usize::from(label)] {
                return Err(BinaryError::RepeatedSibling { index, label });
            }
            seen[usize::from(label)] = true;
            let next = units::follow(&self.units, index, label)?
                .ok_or(BinaryError::InvalidTransition { index, label })?;
            payload.push(label);
            self.collect(next, payload, results)?;
            payload.pop();
            label = self.guide[next][1];
        }
        Ok(())
    }
}
