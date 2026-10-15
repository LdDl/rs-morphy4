use crate::dictionary::binary::error::BinaryError;
use crate::dictionary::dawg::records::DawgRecord;
use crate::dictionary::dawg::storage::Storage;
use crate::dictionary::dawg::units;

/// A stored spelling and its records in guide order.
#[derive(Debug)]
pub struct SpellingMatch<R> {
    pub word: String,
    pub records: Vec<R>,
}

struct Branch {
    index: usize,
    offset: usize,
    word: String,
}

impl Storage {
    pub fn lookup_spellings<R: DawgRecord>(
        &self,
        key: &str,
    ) -> Result<Vec<SpellingMatch<R>>, BinaryError> {
        if key.bytes().any(|byte| byte <= 1) {
            return Ok(Vec::new());
        }
        let mut pending = vec![Branch {
            index: 0,
            offset: 0,
            word: String::new(),
        }];
        let mut matches = Vec::new();
        while let Some(branch) = pending.pop() {
            let Some(character) = key[branch.offset..].chars().next() else {
                if let Some(index) = units::follow(&self.units, branch.index, 1)? {
                    let mut records = Vec::new();
                    self.collect(index, &mut Vec::new(), &mut records)?;
                    matches.push(SpellingMatch {
                        word: branch.word,
                        records,
                    });
                }
                continue;
            };
            // The stack visits the original spelling before substitutions.
            let variants = if character == 'е' {
                [Some('ё'), Some('е')]
            } else {
                [None, Some(character)]
            };
            for variant in variants.into_iter().flatten() {
                let mut bytes = [0; 4];
                let mut index = Some(branch.index);
                for byte in variant.encode_utf8(&mut bytes).bytes() {
                    index = match index {
                        Some(current) => units::follow(&self.units, current, byte)?,
                        None => break,
                    };
                }
                if let Some(index) = index {
                    let mut word = branch.word.clone();
                    word.push(variant);
                    pending.push(Branch {
                        index,
                        offset: branch.offset + character.len_utf8(),
                        word,
                    });
                }
            }
        }
        Ok(matches)
    }
}
