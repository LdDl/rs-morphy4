use crate::dictionary::binary::error::BinaryError;

pub const LEAF_BIT: u32 = 1 << 31;

pub fn has_leaf(unit: u32) -> bool {
    unit & (1 << 8) != 0
}

fn offset(unit: u32) -> u32 {
    (unit >> 10) << ((unit & (1 << 9)) >> 6)
}

pub fn unit(units: &[u32], index: usize) -> Result<u32, BinaryError> {
    units
        .get(index)
        .copied()
        .ok_or(BinaryError::UnitOutOfBounds {
            index,
            length: units.len(),
        })
}

pub fn follow(units: &[u32], index: usize, label: u8) -> Result<Option<usize>, BinaryError> {
    let next = index ^ offset(unit(units, index)?) as usize ^ usize::from(label);
    let next_unit = unit(units, next)?;
    if next_unit & (LEAF_BIT | 0xff) == u32::from(label) {
        Ok(Some(next))
    } else {
        Ok(None)
    }
}

pub fn validate_leaf(units: &[u32], index: usize) -> Result<(), BinaryError> {
    let next = index ^ offset(unit(units, index)?) as usize;
    if unit(units, next)? & LEAF_BIT == 0 {
        return Err(BinaryError::InvalidLeaf { index });
    }
    Ok(())
}
